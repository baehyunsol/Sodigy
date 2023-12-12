#![deny(unused_imports)]
//! Command Line Argument Parser
//!
//! I want it to emit Sodigy-style error messages. Let's not use https://crates.io/crates/clap.

use sodigy_span::SpanPoint;

mod error;
mod flag;
mod token;
mod parse;

pub use error::ClapError;
use flag::Flag;
use parse::{into_file, into_tokens};
use token::{Token, TokenKind, TokenValue};

pub fn parse_cli_args() -> Result<CompilerOption, Vec<ClapError>> {
    let (input, file) = into_file();

    let tokens = match into_tokens(&input, SpanPoint::at_file(file, 0)) {
        Ok(tokens) => tokens,
        Err(e) => {
            return Err(e);
        }
    };

    if tokens.len() == 1 {
        match &tokens[0] {
            Token {
                kind: TokenKind::Flag,
                value,
                ..
            } => match value {
                TokenValue::Flag(Flag::Help) => Ok(CompilerOption::help_message()),
                TokenValue::Flag(Flag::Version) => Ok(CompilerOption::version_info()),

                // otherwise, `into_tokens` should have returned `Err(e)`
                _ => unreachable!(),
            },
            Token {
                kind: TokenKind::Path,
                value: TokenValue::Path(path),
                ..
            } => Ok(CompilerOption {
                input_path: path.to_string(),
                ..CompilerOption::default()
            }),

            // otherwise, `into_tokens` should have returned `Err(e)`
            _ => unreachable!(),
        }
    }

    else {
        // do the actual parsing
        todo!()
    }
}

// TODO: how do other compilers deal with multiple input files?
pub struct CompilerOption {
    pub do_not_compile_and_print_this: Option<SpecialOutput>,
    input_path: String,
    output_path: String,
    format_from: IrStage,
    format_to: IrStage,
    show_warnings: bool,
    save_ir: bool,
    dump_hir: bool,
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
}

impl Default for CompilerOption {
    fn default() -> Self {
        CompilerOption {
            do_not_compile_and_print_this: None,
            input_path: String::new(),
            output_path: String::from("./a.out"),
            format_from: IrStage::Code,

            // TODO: it has to be IrStage::Binary, but that's not implemented yet
            format_to: IrStage::HighIr,
            show_warnings: true,
            save_ir: true,
            dump_hir: false,
        }
    }
}

pub enum IrStage {
    Code, Tokens, HighIr,
}

pub enum SpecialOutput {
    HelpMessage,
    VersionInfo,
}
