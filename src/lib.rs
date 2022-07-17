use std::fs::read_to_string;

use actix_web::{http::header::ContentType, web, HttpResponse, Responder};
use log::{debug, error, info, warn};
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::message::{
    MessageType, RoomMessageEventContent, TextMessageEventContent,
};
use matrix_sdk::ruma::events::OriginalSyncMessageLikeEvent;
use matrix_sdk::ruma::TransactionId;
use matrix_sdk::Client;
use serde_derive::{Deserialize, Serialize};
use threema_gateway::IncomingMessage;
use tokio::sync::Mutex;

use threema::types::Message;

use crate::matrix::util::{
    get_threematrix_room_state, set_threematrix_room_state, ThreematrixStateEventContent,
};
use crate::threema::util::{
    convert_group_id_from_readable_string, convert_group_id_to_readable_string,
};
use crate::threema::ThreemaClient;

pub mod errors;
pub mod matrix;
pub mod threema;
pub mod util;

pub struct AppState {
    pub threema_client: ThreemaClient,
    pub matrix_client: Mutex<Client>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreemaConfig {
    pub secret: String,
    pub private_key: String,
    pub gateway_own_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatrixConfig {
    pub homeserver_url: String,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggerConfig {
    pub level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreematrixConfig {
    pub threema: ThreemaConfig,
    pub matrix: MatrixConfig,
    pub logger: Option<LoggerConfig>,
}

impl ThreematrixConfig {
    pub fn new(path: &str) -> ThreematrixConfig {
        let toml_string = read_to_string(path).unwrap();
        return toml::from_str(&toml_string).unwrap();
    }
}

pub async fn threema_incoming_message_handler(
    incoming_message: web::Form<IncomingMessage>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let threema_client = &app_state.threema_client;
    let decrypted_message = threema_client.process_incoming_msg(&incoming_message).await;

    match decrypted_message {
        Ok(message) => match message {
            Message::GroupTextMessage(group_text_msg) => {
                let matrix_client = app_state.matrix_client.lock().await;

                if group_text_msg.text.starts_with("!threematrix") {
                    let split_text: Vec<&str> = group_text_msg.text.split(" ").collect();
                    match split_text.get(1).map(|str| *str) {
                        Some("bind") => {
                            let rooms = matrix_client.joined_rooms();
                            let matrix_room_id = split_text.get(2);

                            if let Some(matrix_room_id) = matrix_room_id {
                                if let Some(room) = rooms.iter().find(|r| r.room_id() == matrix_room_id) {
                                    if let Ok(r) = convert_group_id_to_readable_string(&group_text_msg.group_id) {
                                        let content: ThreematrixStateEventContent = ThreematrixStateEventContent {
                                            threematrix_threema_group_id: r,
                                        };

                                        if let Err(e) = set_threematrix_room_state(content, room).await {
                                            let err_text = format!("Could not set Matrix room state: {}", e);
                                            send_error_message_to_threema_group(threema_client, err_text, group_text_msg.group_id.as_slice(), false).await;
                                        } else {
                                            let succ_text = format!("Group has been successfully bound to Matrix room: {}", matrix_room_id);
                                            if let Err(e) = threema_client.send_group_msg_by_group_id(succ_text.as_str(), group_text_msg.group_id.as_slice()).await {
                                                error!("Threema: Could not send bind text: {}", e)
                                            }
                                        };
                                    } else {
                                        error!("Threema: Group Id not valid!");
                                    }
                                } else {
                                    let err_text = format!("Matrix room not found. Maybe the bot is not invited or the room id has wrong format!");
                                    send_error_message_to_threema_group(threema_client, err_text, group_text_msg.group_id.as_slice(), false).await;
                                }
                            } else {
                                let err_text = format!("Missing Matrix room id!");
                                send_error_message_to_threema_group(threema_client, err_text, group_text_msg.group_id.as_slice(), false).await;
                            }
                        }
                        Some("help") => {
                            let help_txt =
                                r#"To bind this Threema Group to a Matrix Room, please use the command "!threematrix bind !abc123:homeserver.org".
You can find the required room id in your Matrix client. Attention: This is NOT a "human readable" room alias, but an "internal" room id, which consists of random characters."#;
                            if let Err(e) = threema_client.send_group_msg_by_group_id(help_txt, group_text_msg.group_id.as_slice()).await {
                                error!("Threema: Could not send help text: {}", e)
                            }
                        }
                        _ => {
                            let err_text = format!("Command not found! Use *!threematrix help* for more information");
                            send_error_message_to_threema_group(threema_client, err_text, group_text_msg.group_id.as_slice(), false).await;
                        }
                    }
                } else {
                    let content = RoomMessageEventContent::text_plain(
                        group_text_msg.base.push_from_name.unwrap_or("UNKNOWN".to_owned())
                            + ": "
                            + group_text_msg.text.as_str(),
                    );
                    for room in matrix_client.joined_rooms() {
                        match get_threematrix_room_state(&room).await {
                            Ok(None) => debug!(
                                "Matrix: Room {:?} does not have proper room state",
                                &room.display_name().await.unwrap_or(
                                    matrix_sdk::DisplayName::Named("UNKNOWN".to_owned())
                                )
                            ),
                            Ok(Some(state)) => {
                                if let Ok(group_id) =
                                convert_group_id_to_readable_string(&group_text_msg.group_id)
                                {
                                    if state.threematrix_threema_group_id == group_id {
                                        let txn_id = TransactionId::new();
                                        if let Err(e) = room.send(content.clone(), Some(&txn_id)).await {
                                            let err_txt = format!("Could not send message to Matrix room: {}", e);
                                            send_error_message_to_threema_group(threema_client, err_txt, group_text_msg.group_id.as_slice(), true).await;
                                        }
                                    }
                                }
                            }
                            Err(e) => warn!("Matrix: Could not retrieve room state: {}", e),
                        }
                    }
                }
            }
            Message::GroupCreateMessage(group_create_msg) => {
                info!(
                    "Got group create message with members: {:?}",
                    group_create_msg.members
                );
            }
            Message::GroupRenameMessage(group_rename_msg) => {
                info!(
                    "Got group rename message for: {:?}",
                    group_rename_msg.group_name
                );
            }
            _ => {}
        },
        Err(err) => {
            error!("Threema: Incoming Message Error: {}", err);
        }
    }

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(())
}

async fn send_error_message_to_threema_group(threema_client: &ThreemaClient, err_text: String, group_id: &[u8], log_level_error: bool) {
    if log_level_error {
        error!("Threema: {}", err_text);
    } else {
        warn!("Threema: {}", err_text);
    }
    if let Err(e) = threema_client.send_group_msg_by_group_id(err_text.as_str(), group_id).await {
        error!("Threema: Could not send error message: \"{}\". {}", err_text, e)
    }
}

pub async fn matrix_incoming_message_handler(
    event: OriginalSyncMessageLikeEvent<RoomMessageEventContent>,
    room: Room,
    threema_client: Ctx<ThreemaClient>,
    matrix_client: Client,
) -> () {
    match room {
        Room::Joined(room) => {
            if let OriginalSyncMessageLikeEvent {
                content:
                RoomMessageEventContent {
                    msgtype: MessageType::Text(TextMessageEventContent { body: msg_body, .. }),
                    ..
                },
                sender,
                ..
            } = event
            {
                debug!("incoming message: {}", msg_body);

                match get_threematrix_room_state(&room).await {
                    Ok(None) => debug!(
                        "Matrix: Room {:?} does not have proper room state",
                        &room
                            .display_name()
                            .await
                            .unwrap_or(matrix_sdk::DisplayName::Named("UNKNOWN".to_owned()))
                    ),
                    Ok(Some(threematrix_state)) => {
                        let group_id = convert_group_id_from_readable_string(
                            threematrix_state.threematrix_threema_group_id.as_str(),
                        );

                        let member = room.get_member(&sender).await.unwrap().unwrap();
                        let name = member
                            .display_name()
                            .unwrap_or_else(|| member.user_id().as_str());

                        // Filter out messages coming from our own bridge user
                        if sender != matrix_client.user_id().await.unwrap() {
                            if let Ok(group_id) = group_id {
                                if let Err(e) = threema_client
                                    .send_group_msg_by_group_id(
                                        &(name.to_owned() + ": " + &msg_body),
                                        group_id.as_slice(),
                                    )
                                    .await
                                {
                                    error!("Threema: Couldn't send message to Group: {}", e)
                                    // TODO Send response to Matrix channel
                                }
                            }
                        };
                    }
                    Err(e) => debug!("Matrix: Could not retrieve room state: {}", e),
                }
            }
        }
        _ => {}
    }
}
