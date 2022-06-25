pub enum Message {
    GroupTextMessage {
        base: MessageBase,
        text: String,
        group_creator: String,
        group_id: Vec<u8>,
    },
    TextMessage {
        base: MessageBase,
        text: String,
    },
    GroupCreateMessage {
        base: MessageBase,
        group_creator: String,
        group_id: Vec<u8>,
        members: Vec<String>,
    },
    GroupRenameMessage {
        base: MessageBase,
        group_creator: String,
        group_id: Vec<u8>,
        group_name: String,
    },
}

pub struct MessageBase {
    pub from_identity: String,
    pub to_identity: String,
    pub message_id: String,
    pub push_from_name: Option<String>,
    pub date: u64,
}
