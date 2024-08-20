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

mod arg;
mod error;
mod flag;
mod parse;
mod lex;
mod session;
mod warn;

pub use error::ClapError;
pub use flag::Flag;
use parse::{into_file, parse_cli};
pub use session::ClapSession;
pub use stages::IrStage;
use token::{Token, TokenKind, TokenValue};
pub use warn::ClapWarning;

type Path = String;

pub fn parse_cli_args() -> ClapSession {
    let (input, file) = into_file();

    let parsed_flags = match parse_cli(&input, SpanPoint::at_file(file, 0)) {
        Ok(parsed_flags) => parsed_flags,
        Err(e) => {
            return ClapSession::with_errors(e);
        }
    };

    todo!()
}
