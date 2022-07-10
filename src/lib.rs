use std::fs::read_to_string;

use actix_web::{http::header::ContentType, HttpResponse, Responder, web};
use matrix_sdk::Client;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::room::{Room};
use matrix_sdk::ruma::events::OriginalSyncMessageLikeEvent;
use matrix_sdk::ruma::events::room::message::{
    MessageType, RoomMessageEventContent, TextMessageEventContent,
};
use matrix_sdk::ruma::TransactionId;
use serde_derive::{Deserialize, Serialize};
use threema_gateway::IncomingMessage;
use tokio::sync::Mutex;

use threema::types::Message;

use crate::matrix::util::{get_threematrix_room_state, set_threematrix_room_state, ThreematrixStateEventContent};
use crate::threema::ThreemaClient;
use crate::threema::util::{convert_group_id_from_readable_string, convert_group_id_to_readable_string};

pub mod matrix;
pub mod threema;

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
pub struct ThreematrixConfig {
    pub threema: ThreemaConfig,
    pub matrix: MatrixConfig,
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
    let client = &app_state.threema_client;
    let decrypted_message = client.process_incoming_msg(&incoming_message).await;

    match decrypted_message {
        Ok(message) => match message {
            Message::GroupTextMessage(group_text_msg) => {
                let matrix_client = app_state.matrix_client.lock().await;

                if group_text_msg.text.starts_with("!threematrix") {
                    let split_text: Vec<&str> = group_text_msg.text.split(" ").collect();
                    let rooms = matrix_client.joined_rooms();
                    let room = rooms.iter().find(|r| r.room_id() == split_text[1]).unwrap(); //TODO

                    let content: ThreematrixStateEventContent = ThreematrixStateEventContent {
                        threematrix_threema_group_id: convert_group_id_to_readable_string(&group_text_msg.group_id)
                    };
                    set_threematrix_room_state(content, room).await;
                } else {
                    let content = RoomMessageEventContent::text_plain(
                        group_text_msg.base.push_from_name.unwrap()
                            + ": "
                            + group_text_msg.text.as_str(),
                    );
                    for room in matrix_client.joined_rooms() {
                        if let Some(state) = get_threematrix_room_state(&room).await {
                            if state.threematrix_threema_group_id == convert_group_id_to_readable_string(&group_text_msg.group_id) {
                                let txn_id = TransactionId::new();
                                room.send(content.clone(), Some(&txn_id)).await.unwrap();
                            }
                        }
                    }
                }
            }
            Message::GroupCreateMessage(group_create_msg) => {
                println!(
                    "Got group create message with members: {:?}",
                    group_create_msg.members
                );
            }
            Message::GroupRenameMessage(group_rename_msg) => {
                println!(
                    "Got group rename message for: {:?}",
                    group_rename_msg.group_name
                );
            }
            _ => {}
        },
        Err(_) => {} //TODO
    }

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(())
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
                println!("incoming message: {}", msg_body);

                if let Some(threematrix_state) = get_threematrix_room_state(&room).await {
                    println!("state : {:?}", threematrix_state);

                    let group_id = convert_group_id_from_readable_string(threematrix_state.threematrix_threema_group_id.as_str());

                    let member = room.get_member(&sender).await.unwrap().unwrap();
                    let name = member
                        .display_name()
                        .unwrap_or_else(|| member.user_id().as_str());

                    // Filter out messages coming from our own bridge user
                    if sender != matrix_client.user_id().await.unwrap() {
                        if let Ok(group_id) = group_id {
                            threema_client
                                .send_group_msg_by_group_id(
                                    &(name.to_owned() + ": " + &msg_body),
                                    group_id.as_slice(),
                                )
                                .await;
                        }
                    };
                }
            }
        }
        _ => {}
    }
}
