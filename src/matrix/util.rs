use matrix_sdk::room::{Joined};
use matrix_sdk::ruma::events::SyncStateEvent::Original;
use matrix_sdk::ruma::events::macros::EventContent;

use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.threematrix", kind = State, state_key_type = String)]
pub struct ThreematrixStateEventContent {
    pub threematrix_threema_group_id: String,
}

pub async fn set_threematrix_state(threematrix_state: ThreematrixStateEventContent, room: &Joined) {
    room.send_state_event(threematrix_state, "").await.unwrap();
}

pub async fn get_threematrix_state(room: &Joined) -> Option<ThreematrixStateEventContent> {
    let sync_state = room.get_state_event_static("")
        .await.unwrap();
    if let Some(raw) = sync_state {
        let sync_state = raw.deserialize().unwrap();

        if let Original(event) = sync_state {
            return Some(event.content);
        }
    }
    return None;
}
