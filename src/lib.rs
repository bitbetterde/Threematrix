extern crate core;

use std::fs::read_to_string;

use actix_web::{http::header::ContentType, HttpResponse, Responder, web};
use serde_derive::{Deserialize, Serialize};
use threema_gateway::IncomingMessage;

use threema::types::Message;

use crate::threema::ThreemaClient;

pub mod matrix;
pub mod threema;

pub struct AppState {
    pub threema_client: ThreemaClient,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreemaConfig {
    pub secret: String,
    pub private_key: String,
    pub gateway_own_id: String,
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
    let client = &app_state.threema_client;
    let decrypted_message = client.process_incoming_msg(&incoming_message).await;

    match decrypted_message {
        Ok(message) => match message {
            Message::GroupTextMessage(group_text_msg) => {
                client
                    .send_group_msg_by_group_id(
                        &group_text_msg.text,
                        &group_text_msg.group_creator,
                        group_text_msg.group_id.as_slice(),
                    )
                    .await;
            }
            Message::GroupCreateMessage(group_create_msg) => {
                println!("  members: {:?}", group_create_msg.members);
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
