use crate::errors::{BindThreemaGroupToMatrixError, SendToMatrixRoomByThreemaGroupIdError};
use crate::matrix::util::{set_threematrix_room_state, ThreematrixStateEventContent};
use crate::threema::util::convert_group_id_to_readable_string;
use async_trait::async_trait;
use matrix_sdk_appservice::ruma::RoomId;
use matrix_sdk_appservice::AppService;

use super::MatrixClient;

#[async_trait]
impl MatrixClient for AppService {
    async fn send_message_by_threema_group_id(
        &self,
        threema_group_id: &[u8],
        body: &str,
        html_body: &str,
    ) -> Result<(), SendToMatrixRoomByThreemaGroupIdError> {
        todo!()
    }

    async fn bind_threema_group_to_matrix_room(
        &self,
        threema_group_id: &[u8],
        matrix_room_id: &RoomId,
    ) -> Result<(), BindThreemaGroupToMatrixError> {
        let client = self
            .virtual_user(None)
            .await
            .map_err(|e| BindThreemaGroupToMatrixError::MatrixError(e))?;

        client
            .join_room_by_id(matrix_room_id)
            .await
            .map_err(|e| BindThreemaGroupToMatrixError::MatrixError(e))?;

        let room = client.get_joined_room(room_id).unwrap();

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
