use log::{debug, error, info, warn};
use matrix_sdk::{Client, ruma::events::room::member::StrippedRoomMemberEvent};
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::media::MediaThumbnailSize;
use matrix_sdk::room::{Joined, Room};
use matrix_sdk::ruma::{TransactionId, UInt};
use matrix_sdk::ruma::api::client::media::get_content_thumbnail::v3::Method;
use matrix_sdk::ruma::events::OriginalSyncMessageLikeEvent;
use matrix_sdk::ruma::events::room::message::{ImageMessageEventContent, MessageType, RoomMessageEventContent, TextMessageEventContent};
use threema_gateway::encrypt_file_data;
use tokio::time::{Duration, sleep};

use crate::matrix::util::get_threematrix_room_state;
use crate::threema::ThreemaClient;
use crate::threema::util::convert_group_id_from_readable_string;

pub async fn matrix_incoming_message_handler(
    event: OriginalSyncMessageLikeEvent<RoomMessageEventContent>,
    room: Room,
    threema_client: Ctx<ThreemaClient>,
    matrix_client: Client,
) -> () {
    match room {
        Room::Joined(room) => {
            match event {
                OriginalSyncMessageLikeEvent {
                    content:
                    RoomMessageEventContent {
                        msgtype: MessageType::Text(TextMessageEventContent { body: msg_body, .. }),
                        ..
                    },
                    sender,
                    ..
                } =>
                    {
                        debug!("Matrix: Incoming message: {}", msg_body);

                        let sender_member = room.get_member(&sender).await;
                        match sender_member {
                            Ok(Some(sender_member)) => {
                                let sender_name = sender_member
                                    .display_name()
                                    .unwrap_or_else(|| sender_member.user_id().as_str());

                                // Filter out messages coming from our own bridge user
                                if sender != matrix_client.user_id().await.unwrap() {
                                    match get_threematrix_room_state(&room).await {
                                        Ok(None) => {
                                            let err_txt = format!("Room {} does not have proper room state. Have you bound the room to a Threema group?",
                                                                  &room.display_name().await.unwrap_or(matrix_sdk::DisplayName::Named("UNKNOWN".to_owned())));
                                            send_error_message_to_matrix_room(&room, err_txt, false).await;
                                        }
                                        Ok(Some(threematrix_state)) => {
                                            let group_id = convert_group_id_from_readable_string(
                                                threematrix_state.threematrix_threema_group_id.as_str(),
                                            );

                                            if let Ok(group_id) = group_id {
                                                if let Err(e) = threema_client
                                                    .send_group_msg_by_group_id(
                                                        format!("*{}*: {}", sender_name, msg_body).as_str(),
                                                        group_id.as_slice(),
                                                    )
                                                    .await
                                                {
                                                    let err_txt = format!(
                                                        "Couldn't send message to Threema group: {}",
                                                        e
                                                    );
                                                    send_error_message_to_matrix_room(&room, err_txt, true)
                                                        .await;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            let err_txt = format!("Could not retrieve room state: {}", e);
                                            send_error_message_to_matrix_room(&room, err_txt, true).await;
                                        }
                                    }
                                }
                            }
                            _ => {
                                error!("Matrix: Could not resolve room member!");
                            }
                        }
                    }
                OriginalSyncMessageLikeEvent {
                    content:
                    RoomMessageEventContent {
                        msgtype: MessageType::Image(image),
                        ..
                    },
                    sender,
                    ..
                } =>
                    {
                        let sender_member = room.get_member(&sender).await;
                        match sender_member {
                            Ok(Some(sender_member)) => {
                                let sender_name = sender_member
                                    .display_name()
                                    .unwrap_or_else(|| sender_member.user_id().as_str());

                                // Filter out messages coming from our own bridge user
                                if sender != matrix_client.user_id().await.unwrap() {
                                    match get_threematrix_room_state(&room).await {
                                        Ok(None) => {
                                            let err_txt = format!("Room {} does not have proper room state. Have you bound the room to a Threema group?",
                                                                  &room.display_name().await.unwrap_or(matrix_sdk::DisplayName::Named("UNKNOWN".to_owned())));
                                            send_error_message_to_matrix_room(&room, err_txt, false).await;
                                        }
                                        Ok(Some(threematrix_state)) => {
                                            let group_id = convert_group_id_from_readable_string(
                                                threematrix_state.threematrix_threema_group_id.as_str(),
                                            );

                                            if let Ok(group_id) = group_id {
                                                let image_file = matrix_client.get_file(image.clone(), false).await.unwrap().unwrap();
                                                let thumbnail = matrix_client.get_thumbnail(image, MediaThumbnailSize { height: UInt::new(400).unwrap(), width: UInt::new(400).unwrap(), method: Method::Scale }, false).await.unwrap().unwrap();

                                                debug!("Matrix image size: {} bytes", image_file.len());
                                                debug!("Matrix thumbnail size: {} bytes", thumbnail.len());

                                                threema_client.send_group_file_by_group_id(&image_file, Some(&thumbnail), &group_id).await.unwrap();
                                            }
                                        }
                                        Err(e) => {
                                            let err_txt = format!("Could not retrieve room state: {}", e);
                                            send_error_message_to_matrix_room(&room, err_txt, true).await;
                                        }
                                    }
                                }
                            }
                            _ => {
                                error!("Matrix: Could not resolve room member!");
                            }
                        }
                    }
                _ => {}
            }
        }
        _ => {
            // If bot not member of room, ignore incoming message
        }
    }
}


async fn send_error_message_to_matrix_room(room: &Joined, err_txt: String, log_level_err: bool) {
    if log_level_err {
        error!("Matrix: {}", err_txt);
    } else {
        warn!("Matrix: {}", err_txt);
    }

    let content = RoomMessageEventContent::text_plain(err_txt.clone());
    let txn_id = TransactionId::new();

    if let Err(e) = room.send(content, Some(&txn_id)).await {
        error!("Matrix: Could not send error message: \"{}\". {}",err_txt, e)
    }
}

// Source: https://github.com/matrix-org/matrix-rust-sdk/blob/matrix-sdk-0.5.0/crates/matrix-sdk/examples/autojoin.rs
pub async fn on_stripped_state_member(
    room_member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
) {
    if room_member.state_key != client.user_id().await.unwrap() {
        return;
    }

    if let Room::Invited(room) = room {
        debug!("Matrix: Autojoining room {}", room.room_id());
        let mut delay = 2;

        while let Err(err) = room.accept_invitation().await {
            // retry autojoin due to synapse sending invites, before the
            // invited user can join for more information see
            // https://github.com/matrix-org/synapse/issues/4345
            error!("Matrix: Failed to join room {} ({:?}), retrying in {}s",room.room_id(), err, delay);

            sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                error!("Matrix: Can't join room {} ({:?})", room.room_id(), err);
                break;
            }
        }
        info!("Matrix: Successfully joined room {}", room.room_id());
    }
}
