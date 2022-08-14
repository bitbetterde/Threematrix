use crate::errors::{BindThreemaGroupToMatrixError, SendToMatrixRoomByThreemaGroupIdError};
use async_trait::async_trait;
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
        matrix_room_id: &str,
    ) -> Result<(), BindThreemaGroupToMatrixError> {
        todo!()
    }
}
