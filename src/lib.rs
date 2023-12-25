#![deny(unused_imports)]

pub mod result;
pub mod stages;

#[cfg(test)]
mod tests;

use crate::result::ErrorsAndWarnings;
use crate::stages::{hir_from_tokens, parse_file};
use sodigy_clap::{CompilerOption, IrStage, SpecialOutput};
use sodigy_endec::Endec;

// TODO: nicer type for compile results
pub fn run(options: CompilerOption) -> ErrorsAndWarnings {
    if let Some(sp) = options.do_not_compile_and_print_this {
        match sp {
            SpecialOutput::HelpMessage => {
                println!("{COMPILER_HELP_MESSAGE}");
            },
            SpecialOutput::VersionInfo => {
                println!("sodigy {MAJOR_VERSION}.{MINOR_VERSION}.{PATCH_VERSION}");
            },
        }

        return ErrorsAndWarnings::new();
    }

    let mut errors_and_warnings = ErrorsAndWarnings::new();
    let output_format = options.output_format;

    for file_path in options.input_files.iter() {
        let (result, mut errors_and_warnings_) = match output_format {
            IrStage::Tokens => {
                let (r, o) = parse_file(file_path, Some(errors_and_warnings), &options);

                (r.map(|r| Box::new(r) as Box<dyn Endec>), o)
            },
            IrStage::HighIr => {
                let (r, o) = hir_from_tokens(file_path, Some(errors_and_warnings), &options);

                (r.map(|r| Box::new(r) as Box<dyn Endec>), o)
            },
        };

        // TODO: what if there are multiple inputs?
        if let Some(r) = result {
            if let Some(output_path) = &options.output_path {
                if let Err(e) = r.save_to_file(output_path) {
                    errors_and_warnings_.push_error(e.into());
                }
            }
        }

        errors_and_warnings = errors_and_warnings_;
    }

    errors_and_warnings
}

// TODO: remove these functions
// pub fn compile_file(path: String) -> Result<ErrorsAndWarnings, FileError> {
//     let file_session = unsafe { global_file_session() };
//     let file = file_session.register_file(&path)?;

//     Ok(compile(file))
// }

// pub fn compile_input(input: Vec<u8>) -> ErrorsAndWarnings {
//     let file_session = unsafe { global_file_session() };
//     let file = file_session.register_tmp_file(input);

//     compile(file)
// }

// pub fn compile(file_hash: FileHash) -> ErrorsAndWarnings {
//     let (parse_session, errors_and_warnings) = parse_stage(file_hash, None, None);

//     let parse_session = if let Some(parse_session) = parse_session {
//         parse_session
//     } else {
//         return errors_and_warnings;
//     };

//     let (hir_session, errors_and_warnings) = hir_stage(&parse_session, Some(errors_and_warnings), None);
//     drop(parse_session);

//     let hir_session = if let Some(hir_session) = hir_session {
//         hir_session
//     } else {
//         return errors_and_warnings;
//     };

//     // TODO: this is a tmp code for testing
//     {
//         let main = sodigy_intern::intern_string(b"main".to_vec());

//         if let Some(main_func) = hir_session.func_defs.get(&main) {
//             let main_func = main_func.clone();

//             let mut eval_ctxt = HirEvalCtxt::from_session(&hir_session);

//             match eval_hir(&main_func.return_val, &mut eval_ctxt) {
//                 Ok(v) => {
//                     println!("result: {v}");
//                 },
//                 Err(e) => {
//                     println!("result: eval_hir failed: {e:?}");
//                 },
//             }
//         }

//         else {
//             println!("result: no main function");
//         }
//     }

//     errors_and_warnings
// }

pub const COMPILER_HELP_MESSAGE: &str =
"Usage: sodigy [OPTIONS] INPUT

Examples:
    sodigy a.sdg --to tokens -o a.tokens
        It reads `a.sdg` and converts the code into tokens. But it doesn't make an AST.
        It just saves the tokens to `a.tokens`. You can later resume the compilation
        from this stage.

    sodigy a.tokens --to hir -o a.hir
        In the previous example, we paused the compilation before building an AST.
        This option resumes the compilation and generates an HIR.

Options:
    -h, --help                      Display this message
    -v, --version
    -t, --to [tokens|hir]           Specify the type of the output
                                    It tries to infer the output type from the extension of the output.
                                    If the the extension and `-t` doesn't match, `-t` has higher priority.
                                    If there's no other information the default value is hir.
    -o, --output PATH               Write output to <PATH>
    --show-warnings [true|false]    Show warnings messages (default: true)
    --save-ir [true|false]          Save intermediate representations (default: true)
    --dump-tokens [true|false]      Dump tokens to stdout (default: false)
    --dump-tokens-to PATH           If `dump-tokens` is set, the tokens are dumped to <PATH>
                                    instead of stdout. If `dump-tokens` is not set, it doesn't do anything.
    --dump-hir [true|false]         Dump HIR to stdout (default: false)
    --dump-hir-to PATH              If `dump-hir` is set, the HIR is dumped to <PATH>
                                    instead of stdout. If `dump-hir` is not set, it doesn't do anything.
";

pub const MAJOR_VERSION: u8 = 0;
pub const MINOR_VERSION: u8 = 0;
pub const PATCH_VERSION: u8 = 0;
