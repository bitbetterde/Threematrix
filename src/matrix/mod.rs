use futures::io::ReuniteError;
use log::{debug, error, warn};
use matrix_sdk::{Client, Error};
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::TransactionId;
use async_trait::async_trait;
use crate::errors::{BindThreemaGroupToMatrixError, SendToMatrixRoomByThreemaGroupIdError};
use crate::matrix::util::{get_threematrix_room_state, set_threematrix_room_state, ThreematrixStateEventContent};
use crate::threema::util::convert_group_id_to_readable_string;


pub mod util;

#[async_trait]
pub trait MatrixClient {
    async fn send_message_by_threema_group_id(&self, threema_group_id: &[u8], body: &str, html_body: &str) -> Result<(), SendToMatrixRoomByThreemaGroupIdError>;
    async fn bind_threema_group_to_matrix_room(&self, threema_group_id: &[u8], matrix_room_id: &str) -> Result<(), BindThreemaGroupToMatrixError>;
}

#[async_trait]
impl MatrixClient for Client {
    async fn send_message_by_threema_group_id(&self, threema_group_id: &[u8], body: &str, html_body: &str) -> Result<(), SendToMatrixRoomByThreemaGroupIdError> {
        let content = RoomMessageEventContent::text_html(body, html_body);
        let mut room_found = false;
        for room in self.joined_rooms() {
            match get_threematrix_room_state(&room).await {
                Ok(None) => debug!(
                                "Matrix: Room {:?} does not have proper room state",
                                &room.display_name().await.unwrap_or(
                                    matrix_sdk::DisplayName::Named("UNKNOWN".to_owned())
                                )
                            ),
                Ok(Some(state)) => {
                    if let Ok(group_id) = convert_group_id_to_readable_string(&threema_group_id)
                    {
                        if state.threematrix_threema_group_id == group_id {
                            let txn_id = TransactionId::new();
                            room.send(content.clone(), Some(&txn_id)).await
                                .map_err(|e| SendToMatrixRoomByThreemaGroupIdError::MatrixError(e))?;
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

    async fn bind_threema_group_to_matrix_room(&self, threema_group_id: &[u8], matrix_room_id: &str) -> Result<(), BindThreemaGroupToMatrixError> {
        if let Some(room) =
        self.joined_rooms().iter().find(|r| r.room_id() == matrix_room_id)
        {
            match convert_group_id_to_readable_string(&threema_group_id) {
                Ok(r) => {
                    let content: ThreematrixStateEventContent =
                        ThreematrixStateEventContent {
                            threematrix_threema_group_id: r,
                        };

                    if let Err(e) =
                    set_threematrix_room_state(content, room).await
                    {
                        return Err(BindThreemaGroupToMatrixError::MatrixError(e));
                    } else {
                        return Ok(());
                    };
                }
                Err(e) => {
                    return Err(BindThreemaGroupToMatrixError::InvalidGroupId(e));
                }
            }
        } else {
            return Err(BindThreemaGroupToMatrixError::NoRoomForRoomIdFoundError);
        }
    }
}

