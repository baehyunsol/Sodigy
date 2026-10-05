use crate::Error;
use sodigy_endec::Endec;
use sodigy_fs_api::{
    FileError,
    WriteMode,
    create_dir,
    exists,
    join4,
    parent,
    read_bytes,
    write_bytes,
};
use sodigy_stages::Stage;

/// The compiler stores irs (or result) in various places.
/// 1. It can store the output to user-given path.
/// 2. If it has to interpret the bytecodes, it just stores them in memory and directly executes them. (WIP)
/// 3. For incremental compilation, it stores irs in the intermediate_dir.
#[derive(Clone, Debug)]
pub enum StoreIrAt {
    File(String),
    IntermediateDir,
}

#[derive(Clone, Debug)]
pub struct StoreIrOption {
    pub stage: Stage,
    pub at: StoreIrAt,
}

pub fn store_ir_if_has_to<T: Endec>(
    session: &T,
    option: &StoreIrOption,
    finished_stage: Stage,
    content_hash: Option<u128>,
    intermediate_dir: &str,
) -> Result<(), Error> {
    if option.stage != finished_stage {
        return Ok(());
    }

    let content = session.encode();

    match &option.at {
        StoreIrAt::File(s) => {
            write_bytes(s, &content, WriteMode::Atomic)?;
        },
        StoreIrAt::IntermediateDir => {
            let path = join4(
                intermediate_dir,
                "irs",
                &format!("{finished_stage:?}").to_lowercase(),
                &format!(
                    "{}",
                    if let Some(content_hash) = content_hash {
                        format!("{content_hash:x}")
                    } else {
                        String::from("total")
                    },
                ),
            )?;
            let parent = parent(&path)?;

            if !exists(&parent) {
                create_dir(&parent)?;
            }

            write_bytes(
                &path,
                &content,
                WriteMode::Atomic,
            )?;
        },
    }

    Ok(())
}

pub fn get_cached_ir(
    intermediate_dir: &str,
    stage: Stage,
    content_hash: Option<u128>,
) -> Result<Option<Vec<u8>>, FileError> {
    let path = join4(
        intermediate_dir,
        "irs",
        &format!("{stage:?}").to_lowercase(),
        &if let Some(content_hash) = content_hash {
            format!("{content_hash:x}")
        } else {
            String::from("total")
        },
    )?;

    if exists(&path) {
        Ok(Some(read_bytes(&path)?))
    }

    else {
        Ok(None)
    }
}
