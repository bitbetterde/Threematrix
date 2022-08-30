pub mod util;
pub mod errors;
pub mod matrix_client_impl;

use async_trait::async_trait;
use matrix_sdk::room::Joined;
use crate::matrix::errors::{BindThreemaGroupToMatrixError, FindMatrixRoomByThreemaGroupIdError, SendToMatrixRoomByThreemaGroupIdError};


#[async_trait]
pub trait MatrixClient {
    async fn get_joined_room_by_threema_group_id(
        &self,
        threema_group_id: &[u8],
    ) -> Result<Joined, FindMatrixRoomByThreemaGroupIdError>;
    async fn send_message_to_matrix_room(
        &self,
        room: &Joined,
        user_name: &str,
        body: &str,
        html_body: &str,
    ) -> Result<(), SendToMatrixRoomByThreemaGroupIdError>;
    async fn send_file_to_matrix_room(
        &self,
        room: &Joined,
        body: &str,
        file: &[u8],
    ) -> Result<(), SendToMatrixRoomByThreemaGroupIdError>;
    async fn bind_threema_group_to_matrix_room(
        &self,
        threema_group_id: &[u8],
        matrix_room_id: &str,
    ) -> Result<(), BindThreemaGroupToMatrixError>;
}
