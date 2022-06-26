extern crate core;

use std::fs::read_to_string;

use actix_web::{http::header::ContentType, web, HttpResponse, Responder};
use serde_derive::{Deserialize, Serialize};
use threema_gateway::IncomingMessage;

use threema::types::Message;

use crate::threema::ThreemaClient;

pub mod matrix;
pub mod threema;

pub struct AppState {
    pub config: ThreemaConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreemaConfig {
    secret: String,
    private_key: String,
    gateway_from: String,
}

impl ThreemaConfig {
    pub fn new(path: &str) -> ThreemaConfig {
        let toml_string = read_to_string(path).unwrap();
        return toml::from_str(&toml_string).unwrap();
    }
}

pub async fn incoming_message_handler(
    incoming_message: web::Form<IncomingMessage>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let secret = &app_state.config.secret;
    let private_key = &app_state.config.private_key;
    let own_id = &app_state.config.gateway_from;

    let client = ThreemaClient::new(own_id, secret, private_key);

    let decrypted_message = client.process_incoming_msg(&incoming_message).await;

    match decrypted_message {
        Ok(message) => match message {
            Message::GroupTextMessage(group_text_msg) => {
                let members = client.groups.get(&group_text_msg.group_id).unwrap().members;

                if members.len() == 0 {
                    client
                        .send_group_sync_req_msg(
                            &group_text_msg.group_id,
                            &group_text_msg.group_creator,
                        )
                        .await;
                    let mut queued_messages = client.queued_messages;
                    queued_messages.push(group_text_msg);
                } else {
                    let receivers: Vec<&str> = members
                        .iter()
                        .map(|group_id| -> &str { group_id.as_ref() })
                        .collect();

                    client
                        .send_group_msg(
                            &group_text_msg.text,
                            &group_text_msg.group_creator,
                            group_text_msg.group_id.as_slice(),
                            receivers.as_slice(),
                        )
                        .await;
                }
            }
            Message::GroupCreateMessage(group_create_msg) => {
                println!("  members: {:?}", group_create_msg.members);

                let mut members = &client
                    .groups
                    .get(&group_create_msg.group_id)
                    .unwrap()
                    .members;
                *members = group_create_msg
                    .members
                    .iter()
                    .filter(|member| *member != own_id)
                    .cloned()
                    .collect();

                let mut queued_messages = client.queued_messages;
                for message in queued_messages.drain(..) {
                    let receivers: Vec<&str> = members
                        .iter()
                        .map(|group_id| -> &str { group_id.as_ref() })
                        .collect();

                    &client
                        .send_group_msg(
                            &message.text,
                            &message.group_creator,
                            message.group_id.as_slice(),
                            receivers.as_slice(),
                        )
                        .await;
                }
            }
            Message::GroupRenameMessage(group_rename_msg) => {
                println!("group name: {:?}", group_rename_msg.group_name);
            }
            _ => {}
        },
        Err(_) => {} //TODO
    }

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(())
}
