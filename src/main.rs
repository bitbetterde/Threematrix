extern crate serde_derive;
extern crate core;

use std::fs::read_to_string;
use std::iter::repeat;

use actix_web::{App, http::header::ContentType, HttpResponse, HttpServer, post, Responder, web};
use rand::Rng;
use threema_gateway::{ApiBuilder, E2eApi, EncryptedMessage, IncomingMessage, MessageType, RecipientKey};
use serde_derive::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
struct ThreemaConfig {
    secret: String,
    private_key: String,
    gateway_from: String,
    to_user_id_1: String,
    to_user_id_2: String,
}

#[post("/callback")]
async fn incoming_message(incoming_message: web::Form<IncomingMessage>) -> impl Responder {
    let toml_string = read_to_string("../threema_gateway_cfg.toml").unwrap();
    let cfg: ThreemaConfig = toml::from_str(&toml_string).unwrap();

    let secret = cfg.secret;
    let private_key = cfg.private_key;

    let from = cfg.gateway_from;
    let to_group_ids = vec![cfg.to_user_id_1, cfg.to_user_id_2];


    // Create E2eApi instance
    let api = ApiBuilder::new(from, secret)
        .with_private_key_str(private_key.as_ref())
        .and_then(|builder| builder.into_e2e())
        .unwrap();


    println!("Parsed and validated message from request:");
    println!("  From: {}", incoming_message.from);
    println!("  To: {}", incoming_message.to);
    println!("  Message ID: {}", incoming_message.message_id);
    println!("  Timestamp: {}", incoming_message.date);
    println!("  Sender nickname: {:?}", incoming_message.nickname);

    // Fetch sender public key
    let pubkey = api.lookup_pubkey(&incoming_message.from).await.unwrap_or_else(|e| {
        eprintln!("Could not fetch public key for {}: {:?}", &incoming_message.from, e);
        std::process::exit(1);
    });

    // Decrypt
    let data = api
        .decrypt_incoming_message(&incoming_message, &pubkey)
        .unwrap_or_else(|e| {
            println!("Could not decrypt box: {:?}", e);
            std::process::exit(1);
        });

    let message_type: u8 = &data[0] & 0xFF;
    println!("  MessageType: {:#02x}", message_type);
    let msg_type_as_u8: u8 = MessageType::Text.into();
    //GroupTextMessage
    if message_type == 0x41 {
        let group_creator = String::from_utf8(data[1..9].to_vec()).unwrap();
        let group_id = &data[9..17];
        let text = String::from_utf8(data[17..].to_vec()).unwrap();

        // Show result
        println!("  GroupCreator: {}", group_creator);
        println!("  groupId: {:?}", group_id);
        println!("  text: {}", text);

        for user_id in to_group_ids {
            let public_key = api.lookup_pubkey(&user_id).await.unwrap();
            let encrypted_msg = encrypt_group_text_msg(&text, &group_creator, group_id, &public_key.into(), &api);

            match api.send(&user_id, &encrypted_msg, false).await {
                Ok(msg_id) => println!("Sent. Message id is {}.", msg_id),
                Err(e) => println!("Could not send message: {:?}", e),
            }
        }
    } else if message_type == msg_type_as_u8 {
        let text = String::from_utf8(data[1..].to_vec());
        println!("  text: {}", text.unwrap());
    } else {
        println!("  content: {:?}", &data[1..]);
    }


    HttpResponse::Ok().content_type(ContentType::plaintext()).body(())
}

pub fn encrypt_group_text_msg(text: &str, group_creator: &str, group_id: &[u8], recipient_key: &RecipientKey, threema_api: &E2eApi) -> EncryptedMessage {
    let padding_amount = random_padding_amount();
    let padding = repeat(padding_amount).take(padding_amount as usize);
    let msgtype_byte = repeat(0x41).take(1);

    let data: Vec<u8> = group_creator.as_bytes().iter().cloned().chain(group_id.iter().cloned()).chain(text.as_bytes().iter().cloned()).collect();
    let padded_plaintext: Vec<u8> = msgtype_byte
        .chain(data.iter().cloned())
        .chain(padding)
        .collect();

    threema_api.encrypt_raw(&padded_plaintext, &recipient_key)
}

fn random_padding_amount() -> u8 {
    let mut rng = rand::thread_rng();
    return rng.gen_range(1..255);
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Start Server");

    tokio::join!(HttpServer::new(|| {
        App::new()
        .service(incoming_message)
    })
        .bind(("127.0.0.1", 8888))?
        .run());
    Ok(())
}
