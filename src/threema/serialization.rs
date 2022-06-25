use rand::Rng;
use std::iter::repeat;
use threema_gateway::{E2eApi, EncryptedMessage, RecipientKey};

pub fn encrypt_group_text_msg(
    text: &str,
    group_creator: &str,
    group_id: &[u8],
    recipient_key: &RecipientKey,
    threema_api: &E2eApi,
) -> EncryptedMessage {
    let padding_amount = random_padding_amount();
    let padding = repeat(padding_amount).take(padding_amount as usize);
    let msgtype_byte = repeat(0x41).take(1);

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
