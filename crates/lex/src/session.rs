use crate::LexState;
use sodigy_error::{Error, Warning};
use sodigy_file::File;
use sodigy_session::Session as SodigySession;
use sodigy_span::Span;
use sodigy_token::Token;

pub struct Session {
    pub file: File,
    pub(crate) input_bytes: Vec<u8>,
    pub(crate) state: LexState,
    pub(crate) cursor: usize,
    pub tokens: Vec<Token>,
    pub intermediate_dir: String,

    pub(crate) group_stack: Vec<(u8, Span)>,  // u8: b']' | b'}' | b')', Span: span of the opening delim

    // offset of the start of the current token
    pub(crate) token_start: usize,

    // identifier, integer
    pub(crate) buffer1: Vec<u8>,

    // fraction
    pub(crate) buffer2: Vec<u8>,

    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl SodigySession for Session {
    fn get_errors(&self) -> &[Error] {
        &self.errors
    }

    fn get_warnings(&self) -> &[Warning] {
        &self.warnings
    }

    fn get_intermediate_dir(&self) -> &str {
        &self.intermediate_dir
    }
}
