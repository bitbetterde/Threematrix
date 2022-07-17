use crate::errors::{ParseGroupIdError, StringifyGroupIdError};
use crate::threema::GROUP_ID_NUM_BYTES;

pub fn convert_group_id_to_readable_string(group_id: &[u8]) -> Result<String, StringifyGroupIdError> {
    let result = group_id
        .iter()
        .map(|value| format!("{}", value))
        .reduce(|a, b| a + " " + b.as_str());
    if let Some(result) = result {
        return Ok(result);
    }
    return Err(StringifyGroupIdError::EmptyGroupId);
}

pub fn convert_group_id_from_readable_string(group_id_string: &str) -> Result<Vec<u8>, ParseGroupIdError> {
    let group_id_vec: Vec<&str> = group_id_string.split(" ").collect();
    if group_id_vec.len() != GROUP_ID_NUM_BYTES {
        return Err(ParseGroupIdError::InvalidGroupIdLength);
    }

    return group_id_vec.iter().map(|id_part| id_part.parse::<u8>().map_err(|e| ParseGroupIdError::EncodingError(e))).collect();
}
