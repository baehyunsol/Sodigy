#![deny(unused_imports)]

use sodigy_files::{global_file_session, FileError, FileHash};
use sodigy_interpreter::{HirEvalCtxt, eval_hir};

pub mod option;
mod result;
mod stages;

use stages::{
    parse_stage,
    hir_stage,
};

#[cfg(test)]
mod tests;

use result::ErrorsAndWarnings;

pub fn compile_file(path: String) -> Result<ErrorsAndWarnings, FileError> {
    let file_session = unsafe { global_file_session() };
    let file = file_session.register_file(&path)?;

    Ok(compile(file))
}

pub fn compile_input(input: Vec<u8>) -> ErrorsAndWarnings {
    let file_session = unsafe { global_file_session() };
    let file = file_session.register_tmp_file(input);

    compile(file)
}

// TODO: there's no type for compile result yet
pub fn compile(file_hash: FileHash) -> ErrorsAndWarnings {
    let (parse_session, errors_and_warnings) = parse_stage(file_hash, None);

    let parse_session = if let Some(parse_session) = parse_session {
        parse_session
    } else {
        return errors_and_warnings;
    };

    let (hir_session, errors_and_warnings) = hir_stage(&parse_session, Some(errors_and_warnings), None);
    drop(parse_session);

    let hir_session = if let Some(hir_session) = hir_session {
        hir_session
    } else {
        return errors_and_warnings;
    };

    // TODO: this is a tmp code for testing
    {
        let main = sodigy_intern::intern_string(b"main".to_vec());

        if let Some(main_func) = hir_session.func_defs.get(&main) {
            let main_func = main_func.clone();

            let mut eval_ctxt = HirEvalCtxt::from_session(&hir_session);

            match eval_hir(&main_func.return_val, &mut eval_ctxt) {
                Ok(v) => {
                    println!("result: {v}");
                },
                Err(e) => {
                    println!("result: eval_hir failed: {e:?}");
                },
            }
        }

        else {
            println!("result: no main function");
        }
    }

    errors_and_warnings
}

pub const COMPILER_HELP_MESSAGE: &str =
"Usage: sodigy [OPTIONS] INPUT

Options:
    -h, --help                      Display this message
    -v, --version
    -f, --from [code|tokens|hir]    Specify the type of the input (default: code)
    -t, --to [code|tokens|hir]      Specify the type of the output (default: hir)
    -o, --output FILENAME           Write output to <filename> (default: ./a.out)
    --show-warnings [true|false]    Show warnings messages (default: true)
    --save-ir [true|false]          Save intermediate representations (default: true)
    --dump-hir [true|false]         Dump HIR to stdout (default: false)
";

pub const MAJOR_VERSION: u32 = 0;
pub const MINOR_VERSION: u32 = 0;
pub const PATCH_VERSION: u32 = 0;
