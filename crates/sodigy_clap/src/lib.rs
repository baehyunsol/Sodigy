#![deny(unused_imports)]
//! Command Line Argument Parser
//!
//! I want it to emit Sodigy-style error messages. Let's not use [`clap`][clap].
//!
//! [clap]: https://crates.io/crates/clap

use sodigy_config::{
    calc_num_workers,
    CompilerOption,
    CompilerOutputFormat,
    MAX_VERBOSITY,
    MIN_VERBOSITY,
};
use sodigy_span::{SpanPoint, SpanRange};
use std::collections::HashMap;

mod error;
mod flag;
mod parse;
mod session;
mod stages;
mod token;
mod warn;

pub use error::ClapError;
pub use flag::Flag;
use parse::{into_file, into_tokens};
pub use session::ClapSession;
pub use stages::IrStage;
use token::{Token, TokenKind, TokenValue};
pub use warn::ClapWarning;

type Path = String;

// TODO: what if `--help`, `--version` or `--clean` expects more flags?
// I want to set verbosity of `--clean`

// TODO: I want it to warn users when useless `num-workers` is provided. For ex, parser cannot benefit from parallelism

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
                TokenValue::Flag(Flag::Clean) => ClapSession::with_result(
                    CompilerOption::clean_irs()
                ),

                // otherwise, `into_tokens` should have returned `Err(e)`
                _ => unreachable!(),
            },
            Token {
                kind: TokenKind::Path,
                value: TokenValue::Path(path),
                ..
            } => ClapSession::with_result(CompilerOption {
                input_file: Some(path.to_string()),
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
        let mut input_file: Option<(Path, SpanRange)> = None;
        let mut output_path = None;
        let mut stop_at = None;
        let mut show_warnings = None;
        let mut save_ir = None;
        let mut dump_hir_to = None;
        let mut dump_mir_to = None;
        let mut verbosity = None;
        let mut raw_input: Option<(Vec<u8>, SpanRange)> = None;
        let mut num_workers = None;

        let mut help_flag = None;
        let mut version_flag = None;
        let mut clean_flag = None;
        let mut stop_at_flag = None;

        // `into_tokens` guarantees that every flag has a valid argument
        while index < tokens.len() {
            match &tokens[index].kind {
                TokenKind::Path => {
                    if input_file.is_some() {
                        errors.push(ClapError::multiple_input_files(
                            input_file.as_ref().unwrap().1,  // span
                            tokens[index].span,
                        ));
                    }

                    input_file = Some((tokens[index].value.unwrap_path(), tokens[index].span));
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
                        Flag::StopAt => {
                            if stop_at.is_some() {
                                errors.push(ClapError::same_flag_multiple_times(Flag::StopAt, tokens[index].span));
                            }

                            else {
                                stop_at = Some(tokens[index + 1].value.unwrap_stage());
                                stop_at_flag = Some(tokens[index].span.merge(tokens[index + 1].span));
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
                        Flag::DumpHirTo => {
                            if dump_hir_to.is_some() {
                                errors.push(ClapError::same_flag_multiple_times(Flag::DumpHirTo, tokens[index].span));
                            }

                            else {
                                dump_hir_to = Some(tokens[index + 1].value.unwrap_path());
                            }
                        },
                        Flag::DumpMirTo => {
                            if dump_mir_to.is_some() {
                                errors.push(ClapError::same_flag_multiple_times(Flag::DumpMirTo, tokens[index].span));
                            }

                            else {
                                dump_mir_to = Some(tokens[index + 1].value.unwrap_path());
                            }
                        },
                        Flag::Verbose => {
                            if verbosity.is_some() {
                                errors.push(ClapError::same_flag_multiple_times(Flag::Verbose, tokens[index].span));
                            }

                            else {
                                let n = tokens[index + 1].value.unwrap_int();

                                if n > MAX_VERBOSITY as u64 || MIN_VERBOSITY as u64 > n {
                                    errors.push(ClapError::integer_range_error(MIN_VERBOSITY as u64, MAX_VERBOSITY as u64 + 1, n, tokens[index + 1].span));
                                }

                                else {
                                    verbosity = Some(n as u8);
                                }
                            }
                        },
                        Flag::RawInput => {
                            if raw_input.is_some() {
                                errors.push(ClapError::same_flag_multiple_times(Flag::RawInput, tokens[index].span));
                            }

                            else {
                                raw_input = Some((
                                    tokens[index + 1].value.unwrap_raw_input().into_bytes(),
                                    tokens[index].span,
                                ));
                            }
                        },
                        Flag::NumWorkers => {
                            if num_workers.is_some() {
                                errors.push(ClapError::same_flag_multiple_times(Flag::NumWorkers, tokens[index].span));
                            }

                            else {
                                // TODO: what if `as usize` fails?
                                num_workers = Some(tokens[index + 1].value.unwrap_int() as usize);
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
                        Flag::Clean => {
                            if let Some(_) = version_flag {
                                warnings.push(ClapWarning::same_flag_multiple_times(Flag::Clean, tokens[index].span));
                            }

                            else {
                                clean_flag = Some(tokens[index].span);
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

        match (input_file.is_none(), raw_input.is_none()) {
            (true, true) => {
                errors.push(ClapError::no_input_files());
            },
            (false, false) => {
                errors.push(ClapError::multiple_input_files(
                    input_file.as_ref().unwrap().1,
                    raw_input.as_ref().unwrap().1,
                ));
            },
            (true, false)
            | (false, true) => {},
        }

        if let Some(span) = help_flag {
            errors.push(ClapError::unnecessary_flag(Flag::Help, span));
        }

        if let Some(span) = version_flag {
            errors.push(ClapError::unnecessary_flag(Flag::Version, span));
        }

        if let Some(span) = clean_flag {
            errors.push(ClapError::unnecessary_flag(Flag::Clean, span));
        }

        let output = match (output_path, stop_at) {
            (Some(path), Some(_)) => {
                warnings.push(ClapWarning::ignored_flag(Flag::StopAt, stop_at_flag.unwrap(), Flag::Output));

                CompilerOutputFormat::Path(path)
            },
            (Some(path), None) => CompilerOutputFormat::Path(path),
            (None, Some(stop_at)) => if save_ir == Some(false) {
                warnings.push(ClapWarning::ignored_flag(Flag::StopAt, stop_at_flag.unwrap(), Flag::SaveIr));

                CompilerOutputFormat::None
            } else {
                match stop_at {
                    IrStage::HighIr => CompilerOutputFormat::HighIr,
                    IrStage::MidIr => CompilerOutputFormat::MidIr,
                }
            },
            (None, None) => CompilerOutputFormat::None,
        };

        // it not only mutes compiler warnings, but also clap warnings
        if show_warnings == Some(false) {
            warnings.clear();
        }

        let input_file = if let Some((path, _)) = input_file {
            Some(path)
        } else {
            None
        };

        let raw_input = if let Some((bytes, _)) = raw_input {
            Some(bytes)
        } else {
            None
        };

        let comp_option = CompilerOption {
            do_not_compile_and_do_this: None,
            input_file,
            output,
            show_warnings: show_warnings.unwrap_or(true),
            save_ir: save_ir.unwrap_or(true),
            dump_hir_to,
            dump_mir_to,
            dependencies: HashMap::new(),
            verbosity: verbosity.unwrap_or(1),
            raw_input,
            parse_config_file: false,
            num_workers: num_workers.unwrap_or(calc_num_workers()),
        };

        let res = ClapSession {
            result: comp_option,
            errors,
            warnings,
            ..ClapSession::default()
        };

        res
    }
}
