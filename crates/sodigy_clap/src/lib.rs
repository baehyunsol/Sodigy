#![feature(let_chains)]
#![deny(unused_imports)]
//! Command Line Argument Parser
//!
//! I want it to emit Sodigy-style error messages. Let's not use [`clap`][clap].
//!
//! [clap]: https://crates.io/crates/clap

use hmath::BigInt;
use smallvec::smallvec;
use sodigy_config::{
    CompilerOption,
    CompilerOutputFormat,
    MAX_VERBOSITY,
    MIN_VERBOSITY,
    SpecialOutput,
};
use sodigy_session::SodigySession;
use sodigy_span::{SpanPoint, SpanRange};
use std::collections::HashMap;

mod arg;
mod error;
mod flag;
mod parse;
mod lex;
mod session;
mod token;
mod warn;

pub use error::ClapError;
pub use flag::Flag;
use lex::into_file;
use parse::{FlagWithArg, parse_cli};
pub use session::ClapSession;
pub use warn::ClapWarning;

pub fn parse_cli_args() -> ClapSession {
    let (input, file) = into_file();

    let parsed_flags = match parse_cli(
        &input,
        SpanPoint::at_file(file, 7),  // 7 for the string "sodigy "
    ) {
        Ok(parsed_flags) => parsed_flags,
        Err(e) => {
            return ClapSession::with_errors(e);
        }
    };

    // it helps generating errors and warnings
    let mut previous_spans = HashMap::new();
    let mut input_path_span = None;

    let mut errors = vec![];
    let mut warnings = vec![];
    let mut result = CompilerOption::default();

    // `parse_cli` guarantees that args have correct types
    for FlagWithArg {
        flag,
        flag_span,
        arg,
        arg_span,
    } in parsed_flags.into_iter() {
        if let Some(flag) = &flag {
            if let Some(previous_span) = previous_spans.get(flag) {
                errors.push(ClapError::same_flag_multiple_times(
                    Some(*flag),
                    smallvec![
                        *previous_span,
                        flag_span.unwrap(),
                    ],
                ));
                continue;
            }

            previous_spans.insert(*flag, flag_span.unwrap());

            match flag {
                Flag::Verbose => {
                    let verbosity = arg.unwrap().unwrap_int();

                    if verbosity < MIN_VERBOSITY as i64 || verbosity > MAX_VERBOSITY as i64 {
                        errors.push(ClapError::integer_range_error(
                            BigInt::from(MIN_VERBOSITY),
                            BigInt::from(MAX_VERBOSITY).add_i32(1),
                            BigInt::from(verbosity),
                            arg_span.unwrap(),
                        ));
                    }

                    else {
                        result.verbosity = verbosity as u8;
                    }
                },
                Flag::Hir => {
                    result.output_format = CompilerOutputFormat::Hir;
                },
                Flag::Mir => {
                    result.output_format = CompilerOutputFormat::Mir;
                },
                Flag::Output => {
                    result.output_path = Some(arg.unwrap().unwrap_path());
                },
                Flag::Help => {
                    result.do_not_compile_and_do_this = Some(SpecialOutput::HelpMessage);
                },
                Flag::Version => {
                    result.do_not_compile_and_do_this = Some(SpecialOutput::VersionInfo);
                },
                Flag::ShowWarnings => {
                    result.show_warnings = true;
                },
                Flag::HideWarnings => {
                    result.show_warnings = false;
                },
                Flag::RawInput => {
                    result.raw_input = Some(arg.unwrap().unwrap_string().bytes().collect());
                },
                Flag::DumpHirTo => {
                    result.dump_hir_to = Some(arg.unwrap().unwrap_path());
                },
                Flag::DumpMirTo => {
                    result.dump_mir_to = Some(arg.unwrap().unwrap_path());
                },
                Flag::Library => {
                    result.library_paths = Some(arg.unwrap().unwrap_library());
                },
            }
        }

        else {
            let path = arg.unwrap().unwrap_path();

            if let Some(input_path_span) = input_path_span {
                errors.push(ClapError::same_flag_multiple_times(
                    None,
                    smallvec![
                        input_path_span,
                        arg_span.unwrap(),
                    ],
                ));
                continue;
            }

            result.input_path = Some(path);
            input_path_span = arg_span;
        }
    }

    for error in check_incompatible_flags(&previous_spans) {
        errors.push(error);
    }

    for warning in warn_incompatible_flags(&previous_spans) {
        warnings.push(warning);
    }

    if result.do_not_compile_and_do_this.is_none() && result.input_path.is_none() && result.raw_input.is_none() {
        errors.push(ClapError::no_input_file());
    }

    let mut session = if errors.is_empty() {
        ClapSession::with_result(result)
    }

    else {
        ClapSession::with_errors(errors)
    };

    for warning in warnings.into_iter() {
        session.push_warning(warning);
    }

    session
}

fn check_incompatible_flags(flags: &HashMap<Flag, SpanRange>) -> Vec<ClapError> {
    let mut result = vec![];

    if let (Some(hir_span), Some(mir_span)) = (flags.get(&Flag::Hir), flags.get(&Flag::Mir)) {
        result.push(ClapError::incompatible_flags(
            Flag::Hir,
            *hir_span,
            Flag::Mir,
            *mir_span,
        ));
    }

    if let (Some(show_span), Some(hide_span)) = (flags.get(&Flag::ShowWarnings), flags.get(&Flag::HideWarnings)) {
        result.push(ClapError::incompatible_flags(
            Flag::ShowWarnings,
            *show_span,
            Flag::HideWarnings,
            *hide_span,
        ));
    }

    if let Some(help_span) = flags.get(&Flag::Help) && flags.len() > 1 {
        let mut flags_ = flags.clone();
        flags_.remove(&Flag::Help).unwrap();
        let first_flag = flags_.keys().next().unwrap();

        result.push(ClapError::incompatible_flags(
            Flag::Help,
            *help_span,
            *first_flag,
            *flags_.get(first_flag).unwrap(),
        ));
    }

    else if let Some(version_span) = flags.get(&Flag::Version) && flags.len() > 1 {
        let mut flags_ = flags.clone();
        flags_.remove(&Flag::Version).unwrap();
        let first_flag = flags_.keys().next().unwrap();

        result.push(ClapError::incompatible_flags(
            Flag::Version,
            *version_span,
            *first_flag,
            *flags_.get(first_flag).unwrap(),
        ));
    }

    result
}

fn warn_incompatible_flags(flags: &HashMap<Flag, SpanRange>) -> Vec<ClapWarning> {
    let mut result = vec![];

    if let (Some(hir_span), Some(dump_mir_to_span)) = (flags.get(&Flag::Hir), flags.get(&Flag::DumpMirTo)) {
        result.push(ClapWarning::incompatible_flags(
            Flag::Hir,
            *hir_span,
            Flag::DumpMirTo,
            *dump_mir_to_span,
        ));
    }

    if let (Some(hir_span), Some(library_span)) = (flags.get(&Flag::Hir), flags.get(&Flag::Library)) {
        result.push(ClapWarning::incompatible_flags(
            Flag::Hir,
            *hir_span,
            Flag::Library,
            *library_span,
        ));
    }

    result
}
