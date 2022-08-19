use crate::errors::{BindThreemaGroupToMatrixError, SendToMatrixRoomByThreemaGroupIdError};
use crate::matrix::util::{
    get_threematrix_room_state, set_threematrix_room_state, ThreematrixStateEventContent,
};
use crate::threema::util::convert_group_id_to_readable_string;
use async_trait::async_trait;
use log::{debug, warn};
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::{RoomId, TransactionId};
use matrix_sdk::{Client};

use super::MatrixClient;

#[async_trait]
impl MatrixClient for Client {
    async fn send_message_by_threema_group_id(
        &self,
        threema_group_id: &[u8],
        user_name: &str,
        body: &str,
        html_body: &str,
    ) -> Result<(), SendToMatrixRoomByThreemaGroupIdError> {
        let content = RoomMessageEventContent::text_html(
            format!("{}: {}", user_name, body).as_str(),
            format!("<strong>{}</strong>: {}", user_name, html_body));
        let mut room_found = false;
        for room in self.joined_rooms() {
            match get_threematrix_room_state(&room).await {
                Ok(None) => debug!(
                    "Matrix: Room {:?} does not have proper room state",
                    &room
                        .display_name()
                        .await
                        .unwrap_or(matrix_sdk::DisplayName::Named("UNKNOWN".to_owned()))
                ),
                Ok(Some(state)) => {
                    if let Ok(group_id) = convert_group_id_to_readable_string(&threema_group_id) {
                        if state.threematrix_threema_group_id == group_id {
                            let txn_id = TransactionId::new();
                            room.send(content.clone(), Some(&txn_id))
                                .await
                                .map_err(|e| {
                                    SendToMatrixRoomByThreemaGroupIdError::MatrixError(e)
                                })?;
                            room_found = true;
                        }
                    }
                }
                Err(e) => warn!("Matrix: Could not retrieve room state: {}", e),
            }
        }
        return if room_found {
            Ok(())
        } else {
            Err(SendToMatrixRoomByThreemaGroupIdError::NoRoomForGroupIdFoundError)
        };
    }

    async fn bind_threema_group_to_matrix_room(
        &self,
        threema_group_id: &[u8],
        matrix_room_id: &str,
    ) -> Result<(), BindThreemaGroupToMatrixError> {
        let room_id = <&RoomId>::try_from(matrix_room_id)
            .map_err(|e| BindThreemaGroupToMatrixError::InvalidMatrixRoomId(e))?;

        let room = self.get_joined_room(room_id).unwrap();

        match convert_group_id_to_readable_string(&threema_group_id) {
            Ok(r) => {
                let content: ThreematrixStateEventContent = ThreematrixStateEventContent {
                    threematrix_threema_group_id: r,
                };

                if let Err(e) = set_threematrix_room_state(content, &room).await {
                    return Err(BindThreemaGroupToMatrixError::MatrixError(e));
                } else {
                    return Ok(());
                };
            }
            Err(e) => {
                return Err(BindThreemaGroupToMatrixError::InvalidGroupId(e));
            }
        }
    }
}
