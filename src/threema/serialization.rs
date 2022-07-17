use std::iter::repeat;

use rand::Rng;
use threema_gateway::{E2eApi, EncryptedMessage, RecipientKey};

use crate::threema::types::MessageType;

pub fn encrypt_group_sync_req_msg(
    group_id: &[u8],
    recipient_key: &RecipientKey,
    threema_api: &E2eApi,
) -> EncryptedMessage {
    let padding_amount = random_padding_amount();
    let padding = repeat(padding_amount).take(padding_amount as usize);
    let msgtype_byte = repeat(MessageType::GroupRequestSync.into()).take(1);

    let padded_plaintext: Vec<u8> = msgtype_byte
        .chain(group_id.iter().cloned())
        .chain(padding)
        .collect();

    threema_api.encrypt_raw(&padded_plaintext, &recipient_key)
}

pub fn encrypt_group_text_msg(
    text: &str,
    group_creator: &str,
    group_id: &[u8],
    recipient_key: &RecipientKey,
    threema_api: &E2eApi,
) -> EncryptedMessage {
    let padding_amount = random_padding_amount();
    let padding = repeat(padding_amount).take(padding_amount as usize);
    let msgtype_byte = repeat(MessageType::GroupText.into()).take(1);

    let data: Vec<u8> = group_creator
        .as_bytes()
        .iter()
        .cloned()
        .chain(group_id.iter().cloned())
        .chain(text.as_bytes().iter().cloned())
        .collect();
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
