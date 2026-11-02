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
use sodigy_utils::hash;

// There are 256 files in `<ir>/file_map/`. The file names look like `<ir>/file_map/00`,
// `<ir>/file_map/01`, `<ir>/file_map/02`, ... `<ir>/file_map/ff`.
// Each file is a file map.
// A file map is a list of `FileInfo`.
// The name of the file map file is the last 8 bits of file_id, which is same as the last 8 bits of `hash(module_path)`.

fn get_file_map_length(file_map: &[u8]) -> usize {
    todo!()
}

fn encode_file_map(file_map: &[File]) -> Vec<u8> {
    todo!()
}

fn decode_file_map(file_map: &[u8]) -> Result<Vec<File>, FileError> {
    todo!()
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
            let mut file_map: Vec<File> = decode_file_map(&file_map)?;
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

            encode_file_map(&file_map)
        } else {
            encode_file_map(&vec![self.clone()])
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
            todo!()
        } else {
            Ok(None)
        }
    }

    pub(crate) fn search_by_module_path(module_path: &str, intermediate_dir: &str) -> Result<Option<File>, FileError> {
        todo!()
    }
}

