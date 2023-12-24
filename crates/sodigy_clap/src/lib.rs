#![deny(unused_imports)]
//! Command Line Argument Parser
//!
//! I want it to emit Sodigy-style error messages. Let's not use [`clap`][clap].
//!
//! [clap]: https://crates.io/crates/clap

use sodigy_span::SpanPoint;

mod error;
mod flag;
mod parse;
mod session;
mod stages;
mod token;
mod warn;

pub use error::ClapError;
use flag::Flag;
use parse::{into_file, into_tokens};
pub use session::ClapSession;
pub use stages::IrStage;
use token::{Token, TokenKind, TokenValue};
pub use warn::ClapWarning;

pub fn parse_cli_args() -> ClapSession {
    let (input, file) = into_file();

    let tokens = match into_tokens(&input, SpanPoint::at_file(file, 0)) {
        Ok(tokens) => tokens,
        Err(e) => {
            return ClapSession::with_errors(e);
        }
    };

    if tokens.len() == 1 {
        match &tokens[0] {
            Token {
                kind: TokenKind::Flag,
                value,
                ..
            } => match value {
                TokenValue::Flag(Flag::Help) => ClapSession::with_result(
                    CompilerOption::help_message()
                ),
                TokenValue::Flag(Flag::Version) => ClapSession::with_result(
                    CompilerOption::version_info()
                ),

                // otherwise, `into_tokens` should have returned `Err(e)`
                _ => unreachable!(),
            },
            Token {
                kind: TokenKind::Path,
                value: TokenValue::Path(path),
                ..
            } => ClapSession::with_result(CompilerOption {
                input_files: vec![path.to_string()],
                ..CompilerOption::default()
            }),

            // otherwise, `into_tokens` should have returned `Err(e)`
            _ => unreachable!(),
        }
    }

    else {
        let mut index = 0;
        let mut errors = vec![];
        let mut warnings = vec![];
        let mut input_files = vec![];
        let mut output_path = None;
        let mut output_format = None;
        let mut show_warnings = None;
        let mut save_ir = None;
        let mut dump_hir = None;

        let mut help_flag = None;
        let mut version_flag = None;

        // `into_tokens` guarantees that every flag has a valid argument
        while index < tokens.len() {
            match &tokens[index].kind {
                TokenKind::Path => {
                    input_files.push(tokens[index].value.unwrap_path());
                },
                TokenKind::Flag => {
                    match tokens[index].value.unwrap_flag() {
                        Flag::Output => {
                            if output_path.is_some() {
                                errors.push(ClapError::same_flag_multiple_times(Flag::Output, tokens[index].span));
                            }

                            else {
                                output_path = Some(tokens[index + 1].value.unwrap_path());
                            }
                        },
                        Flag::To => {
                            if output_format.is_some() {
                                errors.push(ClapError::same_flag_multiple_times(Flag::To, tokens[index].span));
                            }

                            else {
                                output_format = Some(tokens[index + 1].value.unwrap_stage());
                            }
                        },
                        Flag::ShowWarnings => {
                            if show_warnings.is_some() {
                                errors.push(ClapError::same_flag_multiple_times(Flag::ShowWarnings, tokens[index].span));
                            }

                            else {
                                show_warnings = Some(tokens[index + 1].value.unwrap_bool());
                            }
                        },
                        Flag::SaveIr => {
                            if save_ir.is_some() {
                                errors.push(ClapError::same_flag_multiple_times(Flag::SaveIr, tokens[index].span));
                            }

                            else {
                                save_ir = Some(tokens[index + 1].value.unwrap_bool());
                            }
                        },
                        Flag::DumpHir => {
                            if dump_hir.is_some() {
                                errors.push(ClapError::same_flag_multiple_times(Flag::DumpHir, tokens[index].span));
                            }

                            else {
                                dump_hir = Some(tokens[index + 1].value.unwrap_bool());
                            }
                        },
                        Flag::Help => {
                            if let Some(_) = help_flag {
                                warnings.push(ClapWarning::same_flag_multiple_times(Flag::Help, tokens[index].span));
                            }

                            else {
                                help_flag = Some(tokens[index].span);
                            }
                        },
                        Flag::Version => {
                            if let Some(_) = version_flag {
                                warnings.push(ClapWarning::same_flag_multiple_times(Flag::Version, tokens[index].span));
                            }

                            else {
                                version_flag = Some(tokens[index].span);
                            }
                        },
                    }

                    index += 1;
                },

                // this branch must have been filtered by `into_tokens`
                _ => unreachable!(),
            }

            index += 1;
        }

        if input_files.is_empty() {
            errors.push(ClapError::no_input_files());
        }

        if let Some(span) = help_flag {
            errors.push(ClapError::unnecessary_flag(Flag::Help, span));
        }

        if let Some(span) = version_flag {
            errors.push(ClapError::unnecessary_flag(Flag::Version, span));
        }

        let (output_format, output_path) = match (output_format, output_path) {
            (None, None) => (  // default values
                IrStage::HighIr,
                "a.out".to_string(),
            ),
            (Some(f), None) => {
                // TODO: is `./` okay in Windows?
                let p = format!("./a.{}", f.extension());

                (f, p)
            },
            (None, Some(p)) => {
                let f = if let Some(f) = IrStage::try_infer_from_ext(&p) {
                    f
                } else {
                    // it has to be the last stage among the implemented
                    IrStage::HighIr
                };

                (f, p)
            },
            (Some(f), Some(p)) => {
                if let Some(f_i) = IrStage::try_infer_from_ext(&p) {
                    if f != f_i {
                        warnings.push(ClapWarning::ext_mismatch(f_i, f));
                    }
                }

                (f, p)
            },
        };

        // it not only mutes compiler warnings, but also clap warnings
        if show_warnings == Some(false) {
            warnings.clear();
        }

        let comp_option = CompilerOption {
            do_not_compile_and_print_this: None,
            input_files,
            output_format,
            output_path: Some(output_path),
            show_warnings: show_warnings.unwrap_or(true),
            save_ir: save_ir.unwrap_or(true),
            dump_hir: dump_hir.unwrap_or(false),
        };

        let res = ClapSession {
            result: comp_option,
            errors,
            warnings,
        };

        res
    }
}

pub struct CompilerOption {
    pub do_not_compile_and_print_this: Option<SpecialOutput>,
    pub input_files: Vec<String>,
    pub output_path: Option<String>,
    pub output_format: IrStage,
    pub show_warnings: bool,
    pub save_ir: bool,
    pub dump_hir: bool,
}

impl CompilerOption {
    pub fn help_message() -> Self {
        CompilerOption::print_this_and_quit(SpecialOutput::HelpMessage)
    }

    pub fn version_info() -> Self {
        CompilerOption::print_this_and_quit(SpecialOutput::VersionInfo)
    }

    pub fn print_this_and_quit(s: SpecialOutput) -> Self {
        CompilerOption {
            do_not_compile_and_print_this: Some(s),
            ..CompilerOption::default()
        }
    }

    pub fn test_runner(file_name: &str) -> Self {
        CompilerOption {
            do_not_compile_and_print_this: None,
            input_files: vec![file_name.to_string()],
            output_path: None,

            // TODO: always set it to the latest stage possible
            output_format: IrStage::HighIr,
            show_warnings: true,
            save_ir: false,
            dump_hir: false,
        }
    }
}

impl Default for CompilerOption {
    fn default() -> Self {
        CompilerOption {
            do_not_compile_and_print_this: None,
            input_files: vec![],
            output_path: Some(String::from("./a.out")),

            // TODO: it has to be IrStage::Binary, but that's not implemented yet
            output_format: IrStage::HighIr,
            show_warnings: true,
            save_ir: true,
            dump_hir: false,
        }
    }
}

pub enum SpecialOutput {
    HelpMessage,
    VersionInfo,
}
