use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use threema_gateway::{ApiBuilder, E2eApi, IncomingMessage, PublicKey};
use tokio::sync::Mutex;

use crate::errors::{ProcessIncomingMessageError, SendGroupMessageError};
use log::{debug, info};
use threema_gateway::errors::{ApiBuilderError, ApiError};

use crate::threema::serialization::encrypt_group_sync_req_msg;
use crate::threema::types::{
    GroupCreateMessage, GroupRenameMessage, GroupTextMessage, MessageBase, MessageType, TextMessage,
};
use crate::util::retry_request;

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
    pub fn new(
        own_id: &str,
        secret: &str,
        private_key: &str,
    ) -> Result<ThreemaClient, ApiBuilderError> {
        let api = ApiBuilder::new(own_id, secret)
            .with_private_key_str(private_key.as_ref())
            .and_then(|builder| builder.into_e2e())?;
        return Ok(ThreemaClient {
            api: Arc::new(Mutex::new(api)),
            groups: Arc::new(Mutex::new(HashMap::new())),
        });
    }

    pub async fn send_group_msg_by_group_id(
        &self,
        text: &str,
        group_id: &[u8],
    ) -> Result<(), SendGroupMessageError> {
        let groups = self.groups.lock().await;
        if let Some(group) = groups.get(group_id) {
            let receiver: Vec<&str> = group.members.iter().map(|str| str.as_str()).collect();
            return self
                .send_group_msg(text, &group.group_creator, group_id, receiver.as_slice())
                .await
                .map_err(|e| SendGroupMessageError::ApiError(e));
        } else {
            // TODO warn!("Threema: Could not send message to group, because members are unknown (to be expected, when no Threema message has been received, yet)");
            return Err(SendGroupMessageError::GroupNotInCache);
        }
    }

    pub async fn send_group_msg(
        &self,
        text: &str,
        group_creator: &str,
        group_id: &[u8],
        receivers: &[&str],
    ) -> Result<(), ApiError> {
        let api = self.api.lock().await;
        for user_id in receivers {
            debug!("Threema: Sending message to: {}", user_id);
            let public_key = self.lookup_pubkey_with_retry(user_id, &api).await?; //TODO cache

            let encrypted_msg =
                encrypt_group_text_msg(text, group_creator, group_id, &public_key.into(), &api);

            retry_request(
                || async { api.send(&user_id, &encrypted_msg, false).await },
                20 * 1000,
                6,
            )
            .await?;
            debug!("Threema: Message sent successfully");
        }
        return Ok(());
    }

    async fn lookup_pubkey_with_retry(
        &self,
        user_id: &str,
        api: &E2eApi,
    ) -> Result<PublicKey, ApiError> {
        retry_request(|| async { api.lookup_pubkey(user_id).await }, 20 * 1000, 6).await
    }

    pub async fn send_group_sync_req_msg(
        &self,
        group_id: &[u8],
        receiver: &str,
    ) -> Result<(), ApiError> {
        let api = self.api.lock().await;
        let public_key = self.lookup_pubkey_with_retry(receiver, &api).await?;
        let encrypted_message = encrypt_group_sync_req_msg(group_id, &public_key.into(), &api);

        retry_request(
            || async { api.send(receiver, &encrypted_message, false).await },
            20 * 1000,
            6,
        )
        .await?;
        debug!("Threema: Group sync message sent successfully");
        return Ok(());
    }

    pub async fn process_incoming_msg(
        &self,
        incoming_message: &IncomingMessage,
    ) -> Result<Message, ProcessIncomingMessageError> {
        // debug!("Threema: Parsed and validated message from request: ");
        // debug!("Threema: From: {}", incoming_message.from);
        // debug!("Threema: Sender nickname: {:?}", incoming_message.nickname);
        // debug!("Threema: To: {}", incoming_message.to);
        // debug!("Threema: Timestamp: {}", incoming_message.date);

        let data;
        {
            let api = self.api.lock().await;
            let pubkey = self
                .lookup_pubkey_with_retry(&incoming_message.from, &api)
                .await
                .map_err(|e| ProcessIncomingMessageError::ApiError(e))?;

            data = api
                .decrypt_incoming_message(&incoming_message, &pubkey)
                .map_err(|e| ProcessIncomingMessageError::CryptoError(e))?;
        }
        let message_type: u8 = &data[0] & 0xFF;
        debug!("Threema: Parsed and validated message from request:\nFrom: {}\nSender nickname: {:?}\nTo: {}\nTimestamp: {}\nMessage type: {:#02x}", incoming_message.from,incoming_message.nickname,incoming_message.to,incoming_message.date, message_type);

        let base = MessageBase {
            from_identity: incoming_message.from.clone(),
            to_identity: incoming_message.to.clone(),
            push_from_name: incoming_message.nickname.clone(),
            message_id: incoming_message.message_id.clone(),
            date: incoming_message.date as u64,
        };

        match MessageType::from(message_type) {
            MessageType::Text => {
                let text = String::from_utf8(data[MESSAGE_TYPE_NUM_BYTES..].to_vec())
                    .map_err(|e| ProcessIncomingMessageError::Utf8ConvertError(e))?;
                debug!("Threema: text: {}", text);
                return Ok(Message::TextMessage(TextMessage { base, text }));
            }
            MessageType::GroupText => {
                let group_creator = String::from_utf8(
                    data[MESSAGE_TYPE_NUM_BYTES..MESSAGE_TYPE_NUM_BYTES + GROUP_CREATOR_NUM_BYTES]
                        .to_vec(),
                )
                .map_err(|e| ProcessIncomingMessageError::Utf8ConvertError(e))?;
                let group_id = &data[MESSAGE_TYPE_NUM_BYTES + GROUP_CREATOR_NUM_BYTES
                    ..MESSAGE_TYPE_NUM_BYTES + GROUP_CREATOR_NUM_BYTES + GROUP_ID_NUM_BYTES];
                let text = String::from_utf8(
                    data[MESSAGE_TYPE_NUM_BYTES + GROUP_CREATOR_NUM_BYTES + GROUP_ID_NUM_BYTES..]
                        .to_vec(),
                )
                .map_err(|e| ProcessIncomingMessageError::Utf8ConvertError(e))?;

                // Show result
                debug!(
                    "Threema: GroupCreator: {}\ngroupId: {:?}\ntext: {}",
                    group_creator, group_id, text
                );

                {
                    let groups = self.groups.lock().await;
                    if let None = groups.get(group_id) {
                        debug!("Threema: Unknown group, sending sync req");
                        self.send_group_sync_req_msg(group_id, group_creator.as_str())
                            .await
                            .map_err(|e| ProcessIncomingMessageError::ApiError(e))?;
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
                    current_member_id = current_member_id
                        + String::from_utf8(vec![*char])
                            .map_err(|e| ProcessIncomingMessageError::Utf8ConvertError(e))?
                            .as_str();
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
                    info!("Threema: Leaving group");
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
                let group_id =
                    &data[MESSAGE_TYPE_NUM_BYTES..MESSAGE_TYPE_NUM_BYTES + GROUP_ID_NUM_BYTES];
                let group_name = String::from_utf8(
                    data[MESSAGE_TYPE_NUM_BYTES + GROUP_CREATOR_NUM_BYTES..].to_vec(),
                )
                .map_err(|e| ProcessIncomingMessageError::Utf8ConvertError(e))?;

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
                info!("content: {:?}", &data[1..]);
                Err(ProcessIncomingMessageError::UnknownMessageTypeError)
            }
        }
    }
}
