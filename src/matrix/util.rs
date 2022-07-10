use log::debug;
use matrix_sdk::room::Joined;
use matrix_sdk::ruma::events::macros::EventContent;
use matrix_sdk::ruma::events::SyncStateEvent::Original;

use serde_derive::{Deserialize, Serialize};

use crate::util::retry_request;

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.threematrix", kind = State, state_key_type = String)]
pub struct ThreematrixStateEventContent {
    pub threematrix_threema_group_id: String,
}

pub async fn set_threematrix_room_state(
    threematrix_state: ThreematrixStateEventContent,
    room: &Joined,
) -> Result<(), matrix_sdk::Error> {
    retry_request(
        || async { room.send_state_event(threematrix_state.clone(), "").await },
        20000,
        6,
    )
        .await?;
    debug!("Matrix: Succesfully set room state");
    return Ok(());
}

pub async fn get_threematrix_room_state(
    room: &Joined,
) -> Result<Option<ThreematrixStateEventContent>, matrix_sdk::Error> {
    let sync_state =
        retry_request(|| async { room.get_state_event_static("").await }, 20000, 6).await?;

    if let Some(raw) = sync_state {
        let sync_state = raw.deserialize().unwrap();

        if let Original(event) = sync_state {
            return Ok(Some(event.content));
        }
    }
    return Ok(None);
}
