use sodigy_endec::Endec;
use sodigy_error::{Error, ErrorKind, Warning};
use sodigy_fs_api::{join4, read_bytes};
use sodigy_object_file::{self as object_file, Entry, ObjectFile};

mod rust;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Emit {
    Exe,  // WIP
    ReadableBytecode,
    ExecutableBytecode,
    Rust,  // WIP
    Nothing,
}

pub fn lower(
    object_files: Vec<ObjectFile>,
    mut errors: Vec<Error>,
    warnings: Vec<Warning>,
    emit: Emit,
    intermediate_dir: &str,
) -> (Vec<u8>, Vec<Error>, Vec<Warning>) {
    // TODO: Why not just get a linked object file as an input?
    let linked_object_file = object_file::link(object_files);

    if let Entry::NoEntry = linked_object_file.entry {
        errors.push(Error {
            kind: ErrorKind::CannotFindEntryPoint,
            spans: vec![],
            note: None,
        });
        return (vec![], errors, warnings);
    }

    match emit {
        Emit::Exe => match rust::lower_to_crates(linked_object_file, intermediate_dir, &mut errors) {
            Ok(crate_path) => match sodigy_subprocess::run(
                "cargo",
                &["build", "--release"],
                &crate_path,
                300.0,  // TODO: make it configurable
                false,
                true,
            ) {
                Ok(_) => {
                    let bin_path = join4(&crate_path, "target", "release", "rust-crate").unwrap();

                    match read_bytes(&bin_path) {
                        Ok(bytes) => (bytes, errors, warnings),
                        Err(_) => {
                            errors.push(Error {
                                kind: ErrorKind::InternalCompilerError { id: 339737 },
                                spans: vec![],
                                note: Some(format!("Failed to read bytes at `{bin_path}`.")),
                            });
                            (vec![], errors, warnings)
                        },
                    }
                },
                Err(_) => {
                    errors.push(Error {
                        kind: ErrorKind::InternalCompilerError { id: 336757 },
                        spans: vec![],
                        note: Some(String::from("Failed to build rust crates with cargo.")),
                    });
                    (vec![], errors, warnings)
                },
            },
            Err(()) => (vec![], errors, warnings),
        },
        Emit::ReadableBytecode => (
            linked_object_file.to_string().into_bytes(),
            errors,
            warnings,
        ),
        Emit::ExecutableBytecode => (
            linked_object_file.encode(),
            errors,
            warnings,
        ),
        Emit::Rust => {
            let code = rust::lower_to_single_module(linked_object_file).to_string().into_bytes();
            (code, errors, warnings)
        },
        Emit::Nothing => (vec![], errors, warnings),
    }
}

