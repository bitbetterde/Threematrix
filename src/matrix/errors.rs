use thiserror::Error;
use matrix_sdk::{Error};
use matrix_sdk::ruma::IdParseError;
use crate::errors::StringifyGroupIdError;

#[derive(Debug, Error)]
pub enum FindMatrixRoomByThreemaGroupIdError {
    #[error("No Matrix room for group id found.")]
    NoRoomForGroupIdFoundError,
}

#[derive(Debug, Error)]
pub enum SendToMatrixRoomByThreemaGroupIdError {
    #[error("{0}")]
    MatrixError(Error),
}

#[derive(Debug, Error)]
pub enum BindThreemaGroupToMatrixError {
    #[error("{0}")]
    InvalidGroupId(StringifyGroupIdError),
    #[error("{0}")]
    MatrixError(Error),
    #[error("{0}")]
    InvalidMatrixRoomId(IdParseError),
}