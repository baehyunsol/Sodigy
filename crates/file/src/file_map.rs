use crate::{File, FileId, FileKind};
use sodigy_fs_api::{
    FileError,
    FileErrorKind,
    WriteMode,
    exists,
    join3,
    read_bytes,
    write_bytes,
};
use sodigy_string::{InternedString, intern_string, unintern_string};
use sodigy_utils::hash;

// There are 256 files in `<ir>/file_map/`. The file names look like `<ir>/file_map/00`,
// `<ir>/file_map/01`, `<ir>/file_map/02`, ... `<ir>/file_map/ff`.
// Each file is a file map.
// A file map is a list of `FileInfo`.
// The name of the file map file is the last 8 bits of file_id, which is same as the last 8 bits of `hash(module_path)`.

fn get_file_map_length(file_map: &[u8]) -> usize {
    file_map.len() / 52
}

fn encode_file_map(file_map: &[File], intermediate_dir: &str) -> Result<Vec<u8>, FileError> {
    let mut buffer = Vec::with_capacity(file_map.len() * 52);

    // FIXME: I don't want to intern strings here... but there's no other way!
    for file in file_map.iter() {
        buffer.extend(file.id.id().to_le_bytes());
        buffer.extend(intern_string(file.module_path.as_bytes(), intermediate_dir)?.to_u128().to_le_bytes());
        buffer.extend(file.content.to_u128().to_le_bytes());
        buffer.extend(intern_string(file.file_path.as_bytes(), intermediate_dir)?.to_u128().to_le_bytes());
    }

    Ok(buffer)
}

fn decode_file_map(file_map: &[u8], intermediate_dir: &str) -> Result<Vec<File>, FileError> {
    if file_map.len() % 52 != 0 {
        return Err(FileError {
            kind: FileErrorKind::CannotDecodeFile,
            given_path: None,
        });
    }

    let mut result = Vec::with_capacity(file_map.len() / 52);

    for file in file_map.chunks(52) {
        let id = u32::from_le_bytes(file[..4].try_into().unwrap());
        let module_path = u128::from_le_bytes(file[4..20].try_into().unwrap());
        let content = u128::from_le_bytes(file[20..36].try_into().unwrap());
        let file_path = u128::from_le_bytes(file[36..52].try_into().unwrap());
        result.push(File {
            id: FileId::new(id, FileKind::from_u32(id >> 28).unwrap()),
            module_path: String::from_utf8_lossy(&unintern_string(InternedString::from_u128(module_path), intermediate_dir)?.unwrap()).to_string(),
            content: InternedString::from_u128(content),
            file_path: String::from_utf8_lossy(&unintern_string(InternedString::from_u128(file_path), intermediate_dir)?.unwrap()).to_string(),
        });
    }

    Ok(result)
}

impl FileId {
    pub(crate) fn allocate_or_load_id(module_path: &str, kind: FileKind, intermediate_dir: &str) -> Result<FileId, FileError> {
        match File::search_by_module_path(module_path, intermediate_dir)? {
            Some(file) => Ok(file.id),
            None => {
                let module_path_hash = (hash(module_path.as_bytes()) & 0xff) as u32;
                let file_map_path = join3(
                    intermediate_dir,
                    "file_map",
                    &format!("{module_path_hash:02x}"),
                )?;

                if exists(&file_map_path) {
                    let file_map = read_bytes(&file_map_path)?;
                    let file_map_length = get_file_map_length(&file_map) as u32;
                    Ok(FileId::new((file_map_length << 8) | module_path_hash, kind))
                } else {
                    Ok(FileId::new(module_path_hash, kind))
                }
            },
        }
    }
}

impl File {
    pub(crate) fn update(&self, intermediate_dir: &str) -> Result<(), FileError> {
        let file_map_path = join3(
            intermediate_dir,
            "file_map",
            &format!("{:02x}", self.id.0 & 0xff),
        )?;

        let file_map = if exists(&file_map_path) {
            let file_map = read_bytes(&file_map_path)?;
            let mut file_map: Vec<File> = decode_file_map(&file_map, intermediate_dir)?;
            let mut has_match = false;

            for file in file_map.iter_mut() {
                if file.module_path == self.module_path {
                    *file = self.clone();
                    has_match = true;
                    break;
                }
            }

            if !has_match {
                file_map.push(self.clone());
            }

            encode_file_map(&file_map, intermediate_dir)?
        } else {
            encode_file_map(&vec![self.clone()], intermediate_dir)?
        };

        write_bytes(
            &file_map_path,
            &file_map,
            WriteMode::CreateOrTruncate,
        )
    }

    pub(crate) fn search_by_id(id: FileId, intermediate_dir: &str) -> Result<Option<File>, FileError> {
        let file_map_path = join3(
            intermediate_dir,
            "file_map",
            &format!("{:02x}", id.0 & 0xff),
        )?;

        if exists(&file_map_path) {
            let file_map = read_bytes(&file_map_path)?;
            let file_map = decode_file_map(&file_map, intermediate_dir)?;

            for file in file_map.into_iter() {
                if file.id == id {
                    return Ok(Some(file));
                }
            }

            Ok(None)
        } else {
            Ok(None)
        }
    }

    pub(crate) fn search_by_module_path(module_path: &str, intermediate_dir: &str) -> Result<Option<File>, FileError> {
        let module_path_hash = (hash(module_path.as_bytes()) & 0xff) as u32;
        let file_map_path = join3(
            intermediate_dir,
            "file_map",
            &format!("{module_path_hash:02x}"),
        )?;

        if exists(&file_map_path) {
            let file_map = read_bytes(&file_map_path)?;
            let file_map = decode_file_map(&file_map, intermediate_dir)?;

            for file in file_map.into_iter() {
                if file.module_path == module_path {
                    return Ok(Some(file));
                }
            }

            Ok(None)
        } else {
            Ok(None)
        }
    }
}

