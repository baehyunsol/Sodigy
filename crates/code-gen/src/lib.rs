use sodigy_endec::Endec;
use sodigy_error::{Error, ErrorKind, Warning};
use sodigy_object_file::{self as object_file, Entry, ObjectFile};

mod rust;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Emit {
    Exe,  // WIP
    ReadableBytecode,
    ExecutableBytecode,
    Rust,  // WIP
}

pub fn lower(
    object_files: Vec<ObjectFile>,
    mut errors: Vec<Error>,
    warnings: Vec<Warning>,
    emit: Emit,
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
        Emit::Exe => todo!(),
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
            let code = rust::lower(linked_object_file).code.into_bytes();
            (code, errors, warnings)
        },
    }
}

