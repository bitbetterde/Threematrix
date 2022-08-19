use crate::errors::{BindThreemaGroupToMatrixError, SendToMatrixRoomByThreemaGroupIdError};
use async_trait::async_trait;
use matrix_sdk_appservice::ruma::RoomId;

pub mod util;
pub mod matrix_client_app_service_impl;
pub mod matrix_client_user_impl;

#[async_trait]
pub trait MatrixClient {
    async fn send_message_by_threema_group_id(
        &self,
        threema_group_id: &[u8],
        body: &str,
        html_body: &str,
    ) -> Result<(), SendToMatrixRoomByThreemaGroupIdError>;
    async fn bind_threema_group_to_matrix_room(
        &self,
        threema_group_id: &[u8],
        matrix_room_id: &RoomId,
    ) -> Result<(), BindThreemaGroupToMatrixError>;
}

