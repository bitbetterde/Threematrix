extern crate core;

use std::fs::read_to_string;
use std::sync::Mutex;

use actix_web::{http::header::ContentType, HttpResponse, Responder, web};
use serde_derive::{Deserialize, Serialize};
use threema_gateway::IncomingMessage;

use threema::types::Message;

use crate::threema::ThreemaClient;
use crate::threema::types::GroupTextMessage;

pub mod matrix;
pub mod threema;

pub struct AppState {
    pub config: ThreemaConfig,
    pub members: Mutex<Vec<String>>,
    pub group_name: Mutex<String>,
    pub queued_messages: Mutex<Vec<GroupTextMessage>>,
}


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
    app_state: web::Data<AppState>,
) -> impl Responder {
    let secret = &app_state.config.secret;
    let private_key = &app_state.config.private_key;
    let own_id = &app_state.config.gateway_from;
    // let to_group_ids = vec![&app_state.config.to_user_id_1, &app_state.config.to_user_id_2];

    let client = ThreemaClient::new(own_id, secret, private_key);

    let decrypted_message = client.process_incoming_msg(&incoming_message).await;


    match decrypted_message {
        Ok(message) => {
            match message {
                Message::GroupTextMessage(group_text_msg) => {
                    let members = app_state.members.lock().unwrap();
                    if members.len() == 0 {
                        client.send_group_sync_req_msg(&group_text_msg.group_id, &group_text_msg.group_creator).await;
                        let mut queued_messages = app_state.queued_messages.lock().unwrap();
                        queued_messages.push(group_text_msg);
                    } else {
                        let receivers: Vec<&str> = members
                            .iter()
                            .map(|group_id| -> &str { group_id.as_ref() })
                            .collect();

                        client
                            .send_group_msg(&group_text_msg.text, &group_text_msg.group_creator, group_text_msg.group_id.as_slice(), receivers.as_slice())
                            .await;
                    }
                }
                Message::GroupCreateMessage(group_create_msg) => {
                    println!("member: {:?}", group_create_msg.members);
                    let mut members = app_state.members.lock().unwrap();
                    *members = group_create_msg.members.iter().filter(|member| *member != own_id).cloned().collect();

                    let mut queued_messages = app_state.queued_messages.lock().unwrap();
                    for message in queued_messages.drain(..) {
                        let receivers: Vec<&str> = members
                            .iter()
                            .map(|group_id| -> &str { group_id.as_ref() })
                            .collect();

                        client
                            .send_group_msg(&message.text, &message.group_creator, message.group_id.as_slice(), receivers.as_slice())
                            .await;
                    }
                }
                Message::GroupRenameMessage(group_rename_msg) => {
                    println!("group name: {:?}", group_rename_msg.group_name);
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
