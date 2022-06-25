use crate::threema::ThreemaClient;
use actix_web::{http::header::ContentType, web, HttpResponse, Responder};
use serde_derive::{Deserialize, Serialize};
use threema::types::Message;
use std::fs::read_to_string;
use threema_gateway::IncomingMessage;

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
    // let to_group_ids = vec![&cfg.to_user_id_1, &cfg.to_user_id_2];

    let client = ThreemaClient::new(from, secret, private_key);

    let decrypted_message = client.process_incoming_msg(&incoming_message).await;

    // match decrypted_message {
    //     Ok(message) => {
    //         message.
    //         match message {
    //             Message::GroupCreateMessage => {

    //             }
    //         }
    //     }
    // }
    // client
    //     .send_group_msg(&text, &group_creator, group_id, receivers.as_slice())
    //     .await;

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(())
}
