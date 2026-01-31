use crate::{CompileStage, Error};
use sodigy_endec::{DumpSession, Endec};
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

/// The compiler stores irs (or result) in various places.
/// 1. It can store the output to user-given path.
/// 2. If it has to interpret the bytecodes, it just stores them in memory and directly executes them.
/// 3. In a complicated compilation process, it stores irs in the intermediate_dir.
#[derive(Clone, Debug)]
pub enum StoreIrAt {
    File(String),
    IntermediateDir,
}

#[derive(Clone, Debug)]
pub struct EmitIrOption {
    pub stage: CompileStage,
    pub store: StoreIrAt,
    pub human_readable: bool,
}

pub fn emit_irs_if_has_to<T: Endec + DumpSession>(
    session: &T,
    emit_ir_options: &[EmitIrOption],
    finished_stage: CompileStage,
    content_hash: Option<u128>,
    intermediate_dir: &str,
) -> Result<(), Error> {
    let (mut binary, mut human_readable) = (false, false);
    let stores = emit_ir_options.iter().filter(
        |option| option.stage == finished_stage
    ).map(
        |option| {
            if option.human_readable {
                human_readable = true;
            } else {
                binary = true;
            }

            (option.store.clone(), option.human_readable)
        }
    ).collect::<Vec<_>>();
    let binary = if binary {
        Some(session.encode())
    } else {
        None
    };
    let human_readable = if human_readable {
        Some(session.dump_session())
    } else {
        None
    };

    for (store, human_readable_) in stores.iter() {
        let content = if *human_readable_ {
            human_readable.as_ref().unwrap()
        } else {
            binary.as_ref().unwrap()
        };
        let ext = if *human_readable_ { ".rs" } else { "" };

        match store {
            StoreIrAt::File(s) => {
                write_bytes(&s, content, WriteMode::Atomic)?;
            },
            StoreIrAt::IntermediateDir => {
                let path = join4(
                    intermediate_dir,
                    "irs",
                    &format!("{finished_stage:?}").to_lowercase(),
                    &format!(
                        "{}{ext}",
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
                    content,
                    WriteMode::Atomic,
                )?;
            },
        }
    }

    Ok(())
}

pub fn get_cached_ir(
    intermediate_dir: &str,
    stage: CompileStage,
    content_hash: Option<u128>,
) -> Result<Option<Vec<u8>>, FileError> {
    let path = join4(
        intermediate_dir,
        "irs",
        &format!("{stage:?}").to_lowercase(),
        // There's no `ext` because it's always `!human_readable`
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
