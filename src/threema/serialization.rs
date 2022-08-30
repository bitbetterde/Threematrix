use std::iter::repeat;

use rand::Rng;
use threema_gateway::{E2eApi, EncryptedMessage, FileMessage, RecipientKey};

use crate::threema::types::MessageType;

pub fn encrypt_group_sync_req_msg(
    group_id: &[u8],
    recipient_key: &RecipientKey,
    threema_api: &E2eApi,
) -> EncryptedMessage {
    return threema_api.encrypt(group_id, threema_gateway::MessageType::Other(MessageType::GroupRequestSync.into()), &recipient_key);
}

pub fn encrypt_group_text_msg(
    text: &str,
    group_creator: &str,
    group_id: &[u8],
    recipient_key: &RecipientKey,
    threema_api: &E2eApi,
) -> EncryptedMessage {
    let data: Vec<u8> = group_creator
        .as_bytes()
        .iter()
        .cloned()
        .chain(group_id.iter().cloned())
        .chain(text.as_bytes().iter().cloned())
        .collect();

    return threema_api.encrypt(data.as_slice(), threema_gateway::MessageType::Other(MessageType::GroupText.into()), &recipient_key);
}

pub fn encrypt_group_file_msg(
    msg: &FileMessage,
    group_creator: &str,
    group_id: &[u8],
    recipient_key: &RecipientKey,
    threema_api: &E2eApi,
) -> EncryptedMessage {
    let file_msg_json = serde_json::to_string(msg).unwrap();
    let data: Vec<u8> = group_creator
        .as_bytes()
        .iter()
        .cloned()
        .chain(group_id.iter().cloned())
        .chain(file_msg_json.as_bytes().iter().cloned())
        .collect();

    return threema_api.encrypt(data.as_slice(), threema_gateway::MessageType::Other(MessageType::GroupFile.into()), &recipient_key);
}
