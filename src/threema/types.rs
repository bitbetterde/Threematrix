pub enum Message {
    GroupTextMessage(GroupTextMessage),
    TextMessage(TextMessage),
    GroupCreateMessage(GroupCreateMessage),
    GroupRenameMessage(GroupRenameMessage),
}

pub struct GroupRenameMessage {
    pub base: MessageBase,
    pub group_creator: String,
    pub group_id: Vec<u8>,
    pub group_name: String,
}

pub struct GroupCreateMessage {
    pub base: MessageBase,
    pub group_creator: String,
    pub group_id: Vec<u8>,
    pub members: Vec<String>,
}

pub struct TextMessage {
    pub base: MessageBase,
    pub text: String,
}

pub struct GroupTextMessage {
    pub base: MessageBase,
    pub text: String,
    pub group_creator: String,
    pub group_id: Vec<u8>,
}

pub struct MessageBase {
    pub from_identity: String,
    pub to_identity: String,
    pub message_id: String,
    pub push_from_name: Option<String>,
    pub date: u64,
}
