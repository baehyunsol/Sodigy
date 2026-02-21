mod dist;
mod error;
mod file_size;
mod parser;
mod span;

pub use error::Error;
pub use parser::{ArgCount, ArgFlag, ArgParser, ArgType, Flag, ParsedArgs};
pub use span::underline_span;
