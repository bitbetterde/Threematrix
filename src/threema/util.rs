use crate::threema::GROUP_ID_NUM_BYTES;

pub fn convert_group_id_to_readable_string(group_id: &[u8]) -> String {
    group_id
        .iter()
        .map(|value| format!("{}", value))
        .reduce(|a, b| a + " " + b.as_str()).unwrap()
}

pub fn convert_group_id_from_readable_string(group_id_string: &str) -> Result<Vec<u8>, String> {
    let group_id_vec: Vec<&str> = group_id_string.split(" ").collect();
    if group_id_vec.len() != GROUP_ID_NUM_BYTES {
        return Err("Invalid group id format".to_string());
    }

    Ok(group_id_vec.iter().map(|id_part| id_part.parse::<u8>().unwrap()).collect())
}
