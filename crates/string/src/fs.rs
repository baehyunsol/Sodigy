use crate::InternedString;
use sodigy_fs_api::{
    FileError,
    WriteMode,
    exists,
    join,
    read_bytes,
    write_bytes,
};
use std::fs::File;

pub fn insert_fs_map(dir: &str, id: InternedString, s: &[u8]) -> Result<(), FileError> {
    let lock_file_path = join(dir, "lock")?;
    let lock_file = File::open(&lock_file_path).map_err(|e| FileError::from_std(e, &lock_file_path))?;
    lock_file.lock().map_err(|e| FileError::from_std(e, &lock_file_path))?;

    // large strings are stored in a separate file
    if s.len() >= 256 {
        let save_at = join(dir, &format!("{:x}", id.0))?;
        write_bytes(
            &save_at,
            s,
            WriteMode::CreateOrTruncate,
        )?;
    }

    else {
        let prefix = id.0 & 0xff;
        let path = join(dir, &format!("{prefix:x}"))?;

        let mut data = if exists(&path) {
            let bytes = read_bytes(&path)?;
            decode_fs_map(&bytes)?
        } else {
            vec![]
        };

        data.push((id.0, s.to_vec()));
        write_bytes(
            &path,
            &encode_fs_map(&data),
            WriteMode::CreateOrTruncate,
        )?;
    }

    lock_file.unlock().map_err(|e| FileError::from_std(e, &lock_file_path))?;
    Ok(())
}

// giving an invalid `id` is not an `Err()`, it's `Ok(None)`.
pub fn read_fs_map(dir: &str, id: InternedString) -> Result<Option<Vec<u8>>, FileError> {
    let lock_file_path = join(dir, "lock")?;
    let lock_file = File::open(&lock_file_path).map_err(|e| FileError::from_std(e, &lock_file_path))?;
    lock_file.lock().map_err(|e| FileError::from_std(e, &lock_file_path))?;

    let result = if id.length() >= 256 {
        let stored_at = join(dir, &format!("{:x}", id.0))?;

        if exists(&stored_at) {
            read_bytes(&stored_at).map(|b| Some(b))
        }

        else {
            Ok(None)
        }
    }

    else {
        let prefix = id.0 & 0xff;
        let stored_at = join(dir, &format!("{prefix:x}"))?;

        if exists(&stored_at) {
            let bytes = read_bytes(&stored_at)?;
            let fs_map = decode_fs_map(&bytes)?;
            let mut result = Ok(None);

            for (id_, s) in fs_map.into_iter() {
                if id_ == id.0 {
                    result = Ok(Some(s));
                    break;
                }
            }

            result
        }

        else {
            Ok(None)
        }
    };

    lock_file.unlock().map_err(|e| FileError::from_std(e, &lock_file_path))?;
    result
}

fn encode_fs_map(data: &[(u128, Vec<u8>)]) -> Vec<u8> {
    let mut result = vec![];

    for (id, s) in data.iter() {
        // We don't have to encode the first 3 bytes because it's always 0x800.
        result.extend(&id.to_be_bytes()[3..]);
        result.extend(s);
    }

    result
}

fn decode_fs_map(bytes: &[u8]) -> Result<Vec<(u128, Vec<u8>)>, FileError> {
    let mut result = vec![];
    let mut cursor = 0;

    loop {
        let s_length = match bytes.get(cursor) {
            Some(l) => *l as usize,
            None => {
                break;
            },
        };
        cursor += 1;

        let s_id = match bytes.get(cursor..(cursor + 12)) {
            Some(id) => id.to_vec(),
            None => {
                return Err(/* TODO: Error::CorruptedFile */);
            },
        };
        cursor += 12;

        let s_content = match bytes.get(cursor..(cursor + s_length)) {
            Some(c) => c.to_vec(),
            None => {
                return Err(/* TODO: Error::CorruptedFile */);
            },
        };
        cursor += s_length;

        result.push((u128::from_be_bytes([
            128, 0, 0, s_length as u8,
            s_id[0], s_id[1], s_id[2], s_id[3],
            s_id[4], s_id[5], s_id[6], s_id[7],
            s_id[8], s_id[9], s_id[10], s_id[11],
        ]), s_content));
    }

    Ok(result)
}
