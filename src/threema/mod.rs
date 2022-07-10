use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use threema_gateway::{ApiBuilder, E2eApi, IncomingMessage};
use tokio::sync::Mutex;

use log::{info, debug, warn, error};

use crate::threema::serialization::encrypt_group_sync_req_msg;
use crate::threema::types::{
    GroupCreateMessage, GroupRenameMessage, GroupTextMessage, MessageBase, MessageType, TextMessage,
};

use self::serialization::encrypt_group_text_msg;
use self::types::{Message, MessageGroup};

pub mod serialization;
pub mod types;
pub mod util;

#[derive(Clone)]
pub struct ThreemaClient {
    api: Arc<Mutex<E2eApi>>,
    groups: Arc<Mutex<HashMap<Vec<u8>, MessageGroup>>>,
}

pub const GROUP_ID_NUM_BYTES: usize = 8;
pub const GROUP_CREATOR_NUM_BYTES: usize = 8;
pub const MESSAGE_TYPE_NUM_BYTES: usize = 1;
pub const THREEMA_ID_LENGTH: usize = 8;

impl ThreemaClient {
    pub fn new(own_id: &str, secret: &str, private_key: &str) -> ThreemaClient {
        let api = ApiBuilder::new(own_id, secret)
            .with_private_key_str(private_key.as_ref())
            .and_then(|builder| builder.into_e2e())
            .unwrap();
        return ThreemaClient {
            api: Arc::new(Mutex::new(api)),
            groups: Arc::new(Mutex::new(HashMap::new())),
        };
    }

    pub async fn get_group_id_by_group_name(&self, group_name: &str) -> Option<Vec<u8>> {
        let groups = self.groups.lock().await;
        return groups.iter()
            .find(|group_entry| group_entry.1.name == group_name)
            .map_or(None, |group_entry| Some(group_entry.0.clone()));
    }

    pub async fn send_group_msg_by_group_id(
        &self,
        text: &str,
        group_id: &[u8],
    ) -> () {
        let groups = self.groups.lock().await;
        if let Some(group) = groups.get(group_id) {
            let receiver: Vec<&str> = group.members.iter().map(|str| str.as_str()).collect();
            self.send_group_msg(text, &group.group_creator, group_id, receiver.as_slice()).await;
        } else {
            warn!("Could not send message to Threema group, because members are unknown (to be expected, when no Threema message has been received, yet)");
        }
    }

    pub async fn send_group_msg(
        &self,
        text: &str,
        group_creator: &str,
        group_id: &[u8],
        receivers: &[&str],
    ) -> () {
        let api = self.api.lock().await;
        for user_id in receivers {
            debug!("send msg to:{}", user_id);
            let public_key = api.lookup_pubkey(*user_id).await.unwrap();
            let encrypted_msg =
                encrypt_group_text_msg(text, group_creator, group_id, &public_key.into(), &api);

            match api.send(&user_id, &encrypted_msg, false).await {
                Ok(msg_id) => debug!("Sent. Message id is {}.", msg_id),
                Err(e) => error!("Could not send message: {:?}", e),
            }
        }
    }

    pub async fn send_group_sync_req_msg(&self, group_id: &[u8], receiver: &str) -> () {
        let api = self.api.lock().await;
        let public_key = api.lookup_pubkey(receiver).await.unwrap();
        let encrypted_message = encrypt_group_sync_req_msg(group_id, &public_key.into(), &api);
        match &api.send(receiver, &encrypted_message, false).await {
            Ok(msg_id) => debug!("Sent. Message id is {}.", msg_id),
            Err(e) => error!("Could not send message: {:?}", e),
        }
    }

    pub async fn process_incoming_msg(
        &self,
        incoming_message: &IncomingMessage,
    ) -> Result<Message, ()> {
        debug!("Parsed and validated message from request:");
        debug!("  From: {}", incoming_message.from);
        debug!("  Sender nickname: {:?}", incoming_message.nickname);
        debug!("  To: {}", incoming_message.to);
        debug!("  Message ID: {}", incoming_message.message_id);
        debug!("  Timestamp: {}", incoming_message.date);

        let data;
        {
            let api = self.api.lock().await;
            // Fetch sender public key
            let pubkey = api
                .lookup_pubkey(&incoming_message.from)
                .await
                .unwrap_or_else(|e| {
                    error!(
                        "Could not fetch public key for {}: {:?}",
                        &incoming_message.from, e
                    );
                    std::process::exit(1);
                });

            // Decrypt
            data = api
                .decrypt_incoming_message(&incoming_message, &pubkey)
                .unwrap_or_else(|e| {
                    error!("Could not decrypt box: {:?}", e);
                    std::process::exit(1);
                });
        }
        let message_type: u8 = &data[0] & 0xFF;
        debug!("  MessageType: {:#02x}", message_type);

        let base = MessageBase {
            from_identity: incoming_message.from.clone(),
            to_identity: incoming_message.to.clone(),
            push_from_name: incoming_message.nickname.clone(),
            message_id: incoming_message.message_id.clone(),
            date: incoming_message.date as u64,
        };

        match MessageType::from(message_type) {
            MessageType::Text => {
                let text = String::from_utf8(data[MESSAGE_TYPE_NUM_BYTES..].to_vec()).unwrap();
                debug!("  text: {}", text);
                return Ok(Message::TextMessage(TextMessage { base, text }));
            }
            MessageType::GroupText => {
                let group_creator = String::from_utf8(
                    data[MESSAGE_TYPE_NUM_BYTES..MESSAGE_TYPE_NUM_BYTES + GROUP_CREATOR_NUM_BYTES]
                        .to_vec(),
                )
                    .unwrap();
                let group_id = &data[MESSAGE_TYPE_NUM_BYTES + GROUP_CREATOR_NUM_BYTES
                    ..MESSAGE_TYPE_NUM_BYTES + GROUP_CREATOR_NUM_BYTES + GROUP_ID_NUM_BYTES];
                let text = String::from_utf8(
                    data[MESSAGE_TYPE_NUM_BYTES + GROUP_CREATOR_NUM_BYTES + GROUP_ID_NUM_BYTES..]
                        .to_vec(),
                )
                    .unwrap();

                // Show result
                debug!("  GroupCreator: {}", group_creator);
                debug!("  groupId: {:?}", group_id);
                debug!("  text: {}", text);

                {
                    let groups = self.groups.lock().await;
                    if let None = groups.get(group_id) {
                        debug!("Unknown group, sending sync req");
                        self.send_group_sync_req_msg(group_id, group_creator.as_str())
                            .await;
                    }
                }

                return Ok(Message::GroupTextMessage(GroupTextMessage {
                    base,
                    text,
                    group_creator,
                    group_id: group_id.to_vec(),
                }));
            }
            MessageType::GroupCreate => {
                let group_id =
                    &data[MESSAGE_TYPE_NUM_BYTES..MESSAGE_TYPE_NUM_BYTES + GROUP_CREATOR_NUM_BYTES];
                let mut members: HashSet<String> = HashSet::new();

                let mut counter = 0;
                let mut current_member_id = "".to_owned();
                for char in &data[MESSAGE_TYPE_NUM_BYTES + GROUP_CREATOR_NUM_BYTES..] {
                    current_member_id =
                        current_member_id + String::from_utf8(vec![*char]).unwrap().as_str();
                    counter = counter + 1;
                    if counter == THREEMA_ID_LENGTH {
                        members.insert(current_member_id.clone());

                        current_member_id = "".to_owned();
                        counter = 0;
                    }
                }

                let me_in_group = members
                    .iter()
                    .find(|member| **member == incoming_message.to)
                    .is_some();

                let mut members_without_me: HashSet<&String> = members
                    .iter()
                    .filter(|member| *member != &incoming_message.to)
                    .collect();

                if members_without_me.len() != 0 && me_in_group {
                    // Make sure to always add sender/group creator (different behavior between Android and iOS)
                    members_without_me.insert(&incoming_message.from);

                    {
                        let mut groups = self.groups.lock().await;
                        let new_members: Vec<String> = members_without_me
                            .iter()
                            .map(|member| (*member).to_owned())
                            .collect();
                        groups
                            .entry(group_id.to_vec())
                            .and_modify(|group| {
                                group.members = new_members.clone();
                            })
                            .or_insert(MessageGroup {
                                members: new_members,
                                name: "".to_owned(),
                                group_creator: incoming_message.from.clone(),
                            });

                    }
                } else {
                    let mut groups = self.groups.lock().await;
                    info!("Leaving group");
                    groups.remove(group_id);
                }

                return Ok(Message::GroupCreateMessage(GroupCreateMessage {
                    base,
                    members: members_without_me
                        .iter()
                        .map(|member| (*member).to_owned())
                        .collect(),
                    group_id: group_id.to_vec(),
                }));
            }
            MessageType::GroupRename => {
                let group_id = &data[MESSAGE_TYPE_NUM_BYTES..MESSAGE_TYPE_NUM_BYTES + GROUP_ID_NUM_BYTES];
                let group_name = String::from_utf8(
                    data[MESSAGE_TYPE_NUM_BYTES + GROUP_CREATOR_NUM_BYTES..].to_vec(),
                ).unwrap();

                {
                    let mut groups = self.groups.lock().await;
                    groups
                        .entry(group_id.to_vec())
                        .and_modify(|group| group.name = group_name.clone())
                        .or_insert(MessageGroup {
                            members: Vec::new(),
                            name: group_name.clone(),
                            group_creator: incoming_message.from.clone(),
                        });
                }

                return Ok(Message::GroupRenameMessage(GroupRenameMessage {
                    base,
                    group_name,
                    group_id: group_id.to_vec(),
                }));
            }
            // MessageType::GroupRequestSync => {}
            // MessageType::Image => {}
            // MessageType::Video => {}
            // MessageType::File => {}
            // MessageType::DeliveryReceipt => {}
            _ => {
                info!("Unknown message type received");
                info!("  content: {:?}", &data[1..]);
                Err(())
            }
        }
    }
}
