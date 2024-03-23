use actix_web::{http::header::ContentType, web, HttpResponse, Responder};
use log::{debug, error, info, warn};
use threema_gateway::IncomingMessage;

use crate::{AppState, Message};
use crate::matrix::errors::{BindThreemaGroupToMatrixError, SendToMatrixRoomByThreemaGroupIdError};
use crate::matrix::MatrixClient;
use crate::threema::ThreemaClient;

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
                            let matrix_room_id = split_text.get(2);
                            if let Some(matrix_room_id) = matrix_room_id {
                                match matrix_client.bind_threema_group_to_matrix_room(&group_text_msg.group_id, matrix_room_id).await {
                                    Ok(_) => {
                                        let succ_text = format!("Group has been successfully bound to Matrix room: {}", matrix_room_id);
                                        if let Err(e) = threema_client.send_group_msg_by_group_id(
                                            succ_text.as_str(), group_text_msg.group_id.as_slice()).await
                                        {
                                            error!("Threema: Could not send bind text: {}", e)
                                        }
                                    }
                                    Err(e) => {
                                        match e {
                                            BindThreemaGroupToMatrixError::InvalidGroupId(_) => {
                                                error!("Threema: Group Id not valid!");
                                            }
                                            BindThreemaGroupToMatrixError::MatrixError(e) => {
                                                let err_text = format!("Could not set Matrix room state: {}", e);
                                                send_error_message_to_threema_group(
                                                    threema_client,
                                                    err_text,
                                                    group_text_msg.group_id.as_slice(),
                                                    false,
                                                ).await;
                                            }
                                            BindThreemaGroupToMatrixError::InvalidMatrixRoomId(e) => {
                                                let err_text = format!("Invalid matrix room Id: {}", e);
                                                send_error_message_to_threema_group(
                                                    threema_client,
                                                    err_text,
                                                    group_text_msg.group_id.as_slice(),
                                                    false,
                                                ).await;
                                            }
                                        }
                                    }
                                }
                            } else {
                                let err_text = format!("Missing Matrix room id!");
                                send_error_message_to_threema_group(
                                    threema_client,
                                    err_text,
                                    group_text_msg.group_id.as_slice(),
                                    false,
                                ).await;
                            }
                        }
                        Some("help") => {
                            let help_txt = r#"To bind this Threema Group to a Matrix Room, please use the command "!threematrix bind !abc123:homeserver.org".
You can find the required room id in your Matrix client. Attention: This is NOT a "human readable" room alias, but an "internal" room id, which consists of random characters."#;
                            if let Err(e) = threema_client
                                .send_group_msg_by_group_id(
                                    help_txt,
                                    group_text_msg.group_id.as_slice(),
                                )
                                .await
                            {
                                error!("Threema: Could not send help text: {}", e)
                            }
                        }
                        _ => {
                            let err_text = format!(
                                "Command not found! Use *!threematrix help* for more information"
                            );
                            send_error_message_to_threema_group(
                                threema_client,
                                err_text,
                                group_text_msg.group_id.as_slice(),
                                false,
                            )
                                .await;
                        }
                    }
                } else {
                    let sender_name = group_text_msg
                        .base
                        .push_from_name
                        .unwrap_or("UNKNOWN".to_owned());

                    match matrix_client.get_joined_room_by_threema_group_id(&group_text_msg.group_id).await {
                        Ok(room) => {
                            if let Err(e) = matrix_client
                                .send_message_to_matrix_room(
                                    &room,
                                    sender_name.as_str(),
                                    group_text_msg.text.as_str(),
                                    group_text_msg.text.as_str(),
                                ).await
                            {
                                match e {
                                    SendToMatrixRoomByThreemaGroupIdError::MatrixError(e) => {
                                        let err_txt = format!("Could not send message to Matrix room: {}", e);
                                        send_error_message_to_threema_group(
                                            threema_client,
                                            err_txt,
                                            group_text_msg.group_id.as_slice(),
                                            true,
                                        ).await;
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            debug!("No Matrix room for Threema group id found. Maybe group is not bound to any room");
                        }
                    }
                }
            }
            Message::GroupFileMessage(group_file_msg) => {
                let matrix_client = app_state.matrix_client.lock().await;
                debug!("Threema: Start sending file to Matrix");
                let sender_name = group_file_msg
                    .base
                    .push_from_name
                    .unwrap_or("UNKNOWN".to_owned());

                match matrix_client.get_joined_room_by_threema_group_id(&group_file_msg.group_id).await {
                    Ok(room) => {
                        let file_description = group_file_msg.file_metadata.description().as_deref().unwrap_or("");
                        if let Err(e) = matrix_client
                            .send_message_to_matrix_room(
                                &room,
                                sender_name.as_str(),
                                file_description,
                                file_description,
                            ).await
                        {
                            match e {
                                SendToMatrixRoomByThreemaGroupIdError::MatrixError(e) => {
                                    let err_txt = format!("Could not send message to Matrix room: {}", e);
                                    send_error_message_to_threema_group(
                                        threema_client,
                                        err_txt,
                                        group_file_msg.group_id.as_slice(),
                                        true,
                                    ).await;
                                }
                            }
                        }
                        if let Err(e) = matrix_client
                            .send_file_to_matrix_room(
                                &room,
                                group_file_msg.file_metadata.file_name().as_deref().unwrap_or(""),
                                group_file_msg.file.as_slice(),
                            ).await
                        {
                            match e {
                                SendToMatrixRoomByThreemaGroupIdError::MatrixError(e) => {
                                    let err_txt = format!("Could not send message to Matrix room: {}", e);
                                    send_error_message_to_threema_group(
                                        threema_client,
                                        err_txt,
                                        group_file_msg.group_id.as_slice(),
                                        true,
                                    ).await;
                                }
                            }
                        }
                    }
                    Err(_) => {
                        debug!("No Matrix room for Threema group id found. Maybe group is not bound to any room");
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

async fn send_error_message_to_threema_group(
    threema_client: &ThreemaClient,
    err_text: String,
    group_id: &[u8],
    log_level_error: bool,
) {
    if log_level_error {
        error!("Threema: {}", err_text);
    } else {
        warn!("Threema: {}", err_text);
    }
    if let Err(e) = threema_client
        .send_group_msg_by_group_id(err_text.as_str(), group_id)
        .await
    {
        error!(
            "Threema: Could not send error message: \"{}\". {}",
            err_text, e
        )
    }
}
