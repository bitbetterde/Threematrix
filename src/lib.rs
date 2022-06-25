extern crate core;

use std::fs::read_to_string;

use actix_web::{http::header::ContentType, HttpResponse, Responder, web};
use serde_derive::{Deserialize, Serialize};
use threema_gateway::IncomingMessage;

use threema::types::Message;

use crate::threema::ThreemaClient;

pub mod matrix;
pub mod threema;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreemaConfig {
    secret: String,
    private_key: String,
    gateway_from: String,
    to_user_id_1: String,
    to_user_id_2: String,
}

impl ThreemaConfig {
    pub fn new(path: &str) -> ThreemaConfig {
        let toml_string = read_to_string(path).unwrap();
        return toml::from_str(&toml_string).unwrap();
    }
}

pub async fn incoming_message_handler(
    incoming_message: web::Form<IncomingMessage>,
    cfg: web::Data<ThreemaConfig>,
) -> impl Responder {
    let secret = &cfg.secret;
    let private_key = &cfg.private_key;
    let from = &cfg.gateway_from;
    let to_group_ids = vec![&cfg.to_user_id_1, &cfg.to_user_id_2];

    let client = ThreemaClient::new(from, secret, private_key);

    let decrypted_message = client.process_incoming_msg(&incoming_message).await;


    match decrypted_message {
        Ok(message) => {
            match message {
                Message::GroupTextMessage(group_text_msg) => {
                    let receivers: Vec<&str> = to_group_ids
                        .iter()
                        .map(|group_id| -> &str { group_id.as_ref() })
                        .collect();

                    client
                        .send_group_msg(&group_text_msg.text, &group_text_msg.group_creator, group_text_msg.group_id.as_slice(), receivers.as_slice())
                        .await;
                }
                _ => {}
            }
        }
        Err(_) => {} //TODO
    }

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(())
}
