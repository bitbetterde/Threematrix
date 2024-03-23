use threema_gateway::FileMessage;

// Custom internal types
#[derive(Debug)]
pub struct MessageGroup {
    pub members: Vec<String>,
    pub group_creator: String,
    pub name: String,
}

// Threema types
pub enum Message {
    GroupTextMessage(GroupTextMessage),
    TextMessage(TextMessage),
    GroupFileMessage(GroupFileMessage),
    GroupCreateMessage(GroupCreateMessage),
    GroupRenameMessage(GroupRenameMessage),
}

pub struct GroupRenameMessage {
    pub base: MessageBase,
    pub group_id: Vec<u8>,
    pub group_name: String,
}

pub struct GroupCreateMessage {
    pub base: MessageBase,
    pub group_id: Vec<u8>,
    pub members: Vec<String>,
}

pub struct TextMessage {
    pub base: MessageBase,
    pub text: String,
}

#[derive(Clone)]
pub struct GroupTextMessage {
    pub base: MessageBase,
    pub text: String,
    pub group_creator: String,
    pub group_id: Vec<u8>,
}

pub struct GroupFileMessage {
    pub base: MessageBase,
    pub file_metadata: FileMessage,
    pub group_creator: String,
    pub group_id: Vec<u8>,
    pub file: Vec<u8>,
}

#[derive(Clone)]
pub struct MessageBase {
    pub from_identity: String,
    pub to_identity: String,
    pub message_id: String,
    pub push_from_name: Option<String>,
    pub date: u64,
}


pub enum MessageType {
    Text,
    GroupText,
    GroupFile,
    GroupCreate,
    GroupRename,
    GroupRequestSync,
    Image,
    Video,
    File,
    DeliveryReceipt,
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            0x01 => MessageType::Text,
            0x41 => MessageType::GroupText,
            0x46 => MessageType::GroupFile,
            0x4a => MessageType::GroupCreate,
            0x4b => MessageType::GroupRename,
            0x51 => MessageType::GroupRequestSync,
            0x02 => MessageType::Image,
            0x13 => MessageType::Video,
            0x17 => MessageType::File,
            0x80 => MessageType::DeliveryReceipt,
            _ => {
                panic!("Message type not implemented!");
            }
        }
    }
}

impl Into<u8> for MessageType {
    fn into(self) -> u8 {
        match self {
            MessageType::Text => 0x01,
            MessageType::GroupText => 0x41,
            MessageType::GroupFile => 0x46,
            MessageType::GroupCreate => 0x4a,
            MessageType::GroupRename => 0x4b,
            MessageType::GroupRequestSync => 0x51,
            MessageType::Image => 0x02,
            MessageType::Video => 0x13,
            MessageType::File => 0x17,
            MessageType::DeliveryReceipt => 0x80,
        }
    }
}