use std::io::Cursor;
use log::{debug, warn};
use matrix_sdk::Client;
use matrix_sdk::room::Joined;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::{RoomId, TransactionId};
use crate::matrix::errors::{BindThreemaGroupToMatrixError, FindMatrixRoomByThreemaGroupIdError, SendToMatrixRoomByThreemaGroupIdError};
use crate::matrix::MatrixClient;
use crate::matrix::util::{get_threematrix_room_state, set_threematrix_room_state, ThreematrixStateEventContent};
use crate::threema::util::convert_group_id_to_readable_string;
use async_trait::async_trait;
use matrix_sdk::attachment::AttachmentConfig;

#[async_trait]
impl MatrixClient for Client {
    async fn get_joined_room_by_threema_group_id(&self, threema_group_id: &[u8]) -> Result<Joined, FindMatrixRoomByThreemaGroupIdError> {
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
                            return Ok(room);
                        }
                    }
                }
                Err(e) => warn!("Matrix: Could not retrieve room state: {}", e),
            }
        }
        Err(FindMatrixRoomByThreemaGroupIdError::NoRoomForGroupIdFoundError)
    }

    async fn send_message_to_matrix_room(&self, room: &Joined, user_name: &str, body: &str, html_body: &str) -> Result<(), SendToMatrixRoomByThreemaGroupIdError> {
        let content = RoomMessageEventContent::text_html(
            format!("{}: {}", user_name, body).as_str(),
            format!("<strong>{}</strong>: {}", user_name, html_body));
        let txn_id = TransactionId::new();
        room.send(content.clone(), Some(&txn_id))
            .await
            .map_err(|e| {
                SendToMatrixRoomByThreemaGroupIdError::MatrixError(e)
            })?;
        return Ok(());
    }

    async fn send_file_to_matrix_room(&self, room: &Joined, body: &str, file: &[u8]) -> Result<(), SendToMatrixRoomByThreemaGroupIdError> {
        let mut cursor = Cursor::new(file);
        room.send_attachment(body, &mime::IMAGE_JPEG, &mut cursor, AttachmentConfig::new()).await
            .map_err(|e| { SendToMatrixRoomByThreemaGroupIdError::MatrixError(e) })?;

        return Ok(());
    }

    async fn bind_threema_group_to_matrix_room(&self, threema_group_id: &[u8], matrix_room_id: &str) -> Result<(), BindThreemaGroupToMatrixError> {
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