pub mod util;

use log::{debug, error, info};
use matrix_sdk::{room::Room, ruma::events::room::member::StrippedRoomMemberEvent, Client};
use tokio::time::{sleep, Duration};

// Source: https://github.com/matrix-org/matrix-rust-sdk/blob/matrix-sdk-0.5.0/crates/matrix-sdk/examples/autojoin.rs
pub async fn on_stripped_state_member(
    room_member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
) {
    if room_member.state_key != client.user_id().unwrap() {
        return;
    }

    if let Room::Invited(room) = room {
        debug!("Matrix: Autojoining room {}", room.room_id());
        let mut delay = 2;

        while let Err(err) = room.accept_invitation().await {
            // retry autojoin due to synapse sending invites, before the
            // invited user can join for more information see
            // https://github.com/matrix-org/synapse/issues/4345
            error!(
                "Matrix: Failed to join room {} ({:?}), retrying in {}s",
                room.room_id(),
                err,
                delay
            );

            sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                error!("Matrix: Can't join room {} ({:?})", room.room_id(), err);
                break;
            }
        }
        info!("Matrix: Successfully joined room {}", room.room_id());
    }
}
