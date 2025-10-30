use sodigy_fs_api::{FileError, FileErrorKind};

// A file map is a list of `file_id: u32`, `content_hash: u128`, `module_path: String`.

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
    module_path: &str,
) {
    file_map.extend(file_id.to_le_bytes());
    file_map.extend(content_hash.to_le_bytes());
    file_map.extend((module_path.len() as u32).to_le_bytes());
    file_map.extend(module_path.as_bytes());
}

pub fn search_file_map(file_map: &[u8], module_path: &str, file_map_path: &str) -> Result<Option<(u32, u128)>, FileError> {
    let mut cursor = 0;

    loop {
        if file_map.len() == cursor {
            return Ok(None);
        }

        if cursor + 24 >= file_map.len() {
            return Err(FileError {
                kind: FileErrorKind::CannotDecodeFile,
                given_path: Some(file_map_path.to_string()),
            });
        }

        let file_id = u32::from_le_bytes((&file_map[cursor..(cursor + 4)]).try_into().unwrap());
        cursor += 4;
        let content_hash = u128::from_le_bytes((&file_map[cursor..(cursor + 16)]).try_into().unwrap());
        cursor += 16;
        let str_len = u32::from_le_bytes((&file_map[cursor..(cursor + 4)]).try_into().unwrap());
        cursor += 4;

        if cursor + str_len as usize > file_map.len() {
            return Err(FileError {
                kind: FileErrorKind::CannotDecodeFile,
                given_path: Some(file_map_path.to_string()),
            });
        }

        let curr_normalized_path = &file_map[cursor..(cursor + str_len as usize)];

        if curr_normalized_path == module_path.as_bytes() {
            return Ok(Some((file_id, content_hash)));
        }

        cursor += str_len as usize;
    }
}

pub fn search_file_map_by_id(file_map: &[u8], file_id: u32, file_map_path: &str) -> Result<Option<(String, u128)>, FileError> {
    let mut cursor = 0;

    loop {
        if file_map.len() == cursor {
            return Ok(None);
        }

        if cursor + 24 >= file_map.len() {
            return Err(FileError {
                kind: FileErrorKind::CannotDecodeFile,
                given_path: Some(file_map_path.to_string()),
            });
        }

        let curr_file_id = u32::from_le_bytes((&file_map[cursor..(cursor + 4)]).try_into().unwrap());
        cursor += 4;
        let content_hash = u128::from_le_bytes((&file_map[cursor..(cursor + 16)]).try_into().unwrap());
        cursor += 16;
        let str_len = u32::from_le_bytes((&file_map[cursor..(cursor + 4)]).try_into().unwrap());
        cursor += 4;

        if cursor + str_len as usize > file_map.len() {
            return Err(FileError {
                kind: FileErrorKind::CannotDecodeFile,
                given_path: Some(file_map_path.to_string()),
            });
        }

        let module_path = &file_map[cursor..(cursor + str_len as usize)];

        if curr_file_id == file_id {
            return Ok(Some((String::from_utf8_lossy(module_path).to_string(), content_hash)));
        }

        cursor += str_len as usize;
    }
}
