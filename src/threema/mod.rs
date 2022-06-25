pub mod serialization;
pub mod types;
use crate::threema::types::MessageBase;

use self::serialization::encrypt_group_text_msg;
use self::types::Message;
use threema_gateway::{ApiBuilder, E2eApi, IncomingMessage, MessageType};

pub struct ThreemaClient {
    api: E2eApi,
}

impl ThreemaClient {
    pub fn new(from: &str, secret: &str, private_key: &str) -> ThreemaClient {
        let api = ApiBuilder::new(from, secret)
            .with_private_key_str(private_key.as_ref())
            .and_then(|builder| builder.into_e2e())
            .unwrap();
        return ThreemaClient { api: api };
    }
    pub async fn send_group_msg(
        &self,
        text: &str,
        group_creator: &str,
        group_id: &[u8],
        receivers: &[&str],
    ) -> () {
        for user_id in receivers {
            let public_key = self.api.lookup_pubkey(*user_id).await.unwrap();
            let encrypted_msg = encrypt_group_text_msg(
                text,
                group_creator,
                group_id,
                &public_key.into(),
                &self.api,
            );

            match &self.api.send(&user_id, &encrypted_msg, false).await {
                Ok(msg_id) => println!("Sent. Message id is {}.", msg_id),
                Err(e) => println!("Could not send message: {:?}", e),
            }
        }
    }
    pub async fn process_incoming_msg(
        &self,
        incoming_message: &IncomingMessage,
    ) -> Result<Message, ()> {
        println!("Parsed and validated message from request:");
        println!("  From: {}", incoming_message.from);
        println!("  Sender nickname: {:?}", incoming_message.nickname);
        println!("  To: {}", incoming_message.to);
        println!("  Message ID: {}", incoming_message.message_id);
        println!("  Timestamp: {}", incoming_message.date);

        // Fetch sender public key
        let pubkey = self
            .api
            .lookup_pubkey(&incoming_message.from)
            .await
            .unwrap_or_else(|e| {
                eprintln!(
                    "Could not fetch public key for {}: {:?}",
                    &incoming_message.from, e
                );
                std::process::exit(1);
            });

        // Decrypt
        let data = self
            .api
            .decrypt_incoming_message(&incoming_message, &pubkey)
            .unwrap_or_else(|e| {
                println!("Could not decrypt box: {:?}", e);
                std::process::exit(1);
            });

        let message_type: u8 = &data[0] & 0xFF;
        println!("  MessageType: {:#02x}", message_type);
        let msg_type_as_u8: u8 = MessageType::Text.into();
        // GroupTextMessage
        if message_type == 0x41 {
            let group_creator = String::from_utf8(data[1..9].to_vec()).unwrap();
            let group_id = &data[9..17];
            let text = String::from_utf8(data[17..].to_vec()).unwrap();

            // let receivers: Vec<&str> = to_group_ids
            //     .iter()
            //     .map(|group_id| -> &str { group_id.as_ref() })
            //     .collect();

            // Show result
            println!("  GroupCreator: {}", group_creator);
            println!("  groupId: {:?}", group_id);
            println!("  text: {}", text);

            return Ok(Message::GroupTextMessage {
                base: MessageBase {
                    from_identity: incoming_message.from.clone(),
                    to_identity: incoming_message.to.clone(),
                    push_from_name: incoming_message.nickname.clone(),
                    message_id: incoming_message.message_id.clone(),
                    date: incoming_message.date as u64,
                },
                text: text,
                group_creator: group_creator,
                group_id: group_id.to_vec(),
            });

            // client
            //     .send_group_msg(&text, &group_creator, group_id, receivers.as_slice())
            //     .await;
        } else if message_type == msg_type_as_u8 {
            let text = String::from_utf8(data[1..].to_vec()).unwrap();
            println!("  text: {}", text);
            return Ok(Message::TextMessage {
                base: MessageBase {
                    from_identity: incoming_message.from.clone(),
                    to_identity: incoming_message.to.clone(),
                    push_from_name: incoming_message.nickname.clone(),
                    message_id: incoming_message.message_id.clone(),
                    date: incoming_message.date as u64,
                },
                text: text,
            });
        } else {
            println!("Unknown message type received");
            println!("  content: {:?}", &data[1..]);
            Err(())
        }
    }
}
