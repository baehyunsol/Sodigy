use sodigy_fs_api::{FileError, FileErrorKind};

// A file map is a list of `file_id: u32`, `content_hash: u128`, `normalized_path: String`.

pub fn length_file_map(file_map: &[u8], file_map_path: &str) -> Result<usize, FileError> {
    let mut cursor = 0;
    let mut length = 0;

    loop {
        if file_map.len() == cursor {
            return Ok(length);
        }

        cursor += 20;

        if cursor + 4 >= file_map.len() {
            return Err(FileError {
                kind: FileErrorKind::CannotDecodeFile,
                given_path: Some(file_map_path.to_string()),
            });
        }

        let str_len = u32::from_le_bytes((&file_map[cursor..(cursor + 4)]).try_into().unwrap());
        cursor += 4 + str_len as usize;
        length += 1;
    }
}

pub fn push_file_map(
    file_map: &mut Vec<u8>,
    file_id: u32,
    content_hash: u128,
    normalized_path: &str,
) {
    file_map.extend(file_id.to_le_bytes());
    file_map.extend(content_hash.to_le_bytes());
    file_map.extend(content_hash.to_le_bytes());
    file_map.extend(normalized_path.len().to_le_bytes());
    file_map.extend(normalized_path.as_bytes());
}

pub fn search_file_map(file_map: &[u8], normalized_path: &str, file_map_path: &str) -> Result<Option<(u32, u128)>, FileError> {
    todo!()
}

pub fn search_file_map_by_id(file_map: &[u8], file_id: u32, file_map_path: &str) -> Result<Option<(String, u128)>, FileError> {
    todo!()
}
