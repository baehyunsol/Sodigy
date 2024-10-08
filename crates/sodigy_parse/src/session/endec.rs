use super::ParseSession;
use crate::{ParseError, ParseWarning, TokenTree};
use sodigy_config::CompilerOption;
use sodigy_endec::{
    Endec,
    EndecError,
    EndecSession,
};
use sodigy_error::UniversalError;
use sodigy_intern::{InternedString, InternSession};
use sodigy_session::SessionSnapshot;
use sodigy_span::SpanRange;
use std::collections::HashMap;

impl Endec for ParseSession {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        // there's no point in encoding InternSession

        self.tokens.encode(buffer, session);
        self.errors.encode(buffer, session);
        self.warnings.encode(buffer, session);
        self.unexpanded_macros.encode(buffer, session);
        self.snapshots.encode(buffer, session);
        self.compiler_option.encode(buffer, session);
        self.previous_errors.encode(buffer, session);
        self.previous_warnings.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        // there's no point in decoding InternSession

        Ok(ParseSession {
            tokens: Vec::<TokenTree>::decode(buffer, index, session)?,
            errors: Vec::<ParseError>::decode(buffer, index, session)?,
            warnings: Vec::<ParseWarning>::decode(buffer, index, session)?,
            interner: InternSession::new(),
            unexpanded_macros: HashMap::<InternedString, SpanRange>::decode(buffer, index, session)?,
            snapshots: Vec::<SessionSnapshot>::decode(buffer, index, session)?,
            compiler_option: CompilerOption::decode(buffer, index, session)?,
            previous_errors: Vec::<UniversalError>::decode(buffer, index, session)?,
            previous_warnings: Vec::<UniversalError>::decode(buffer, index, session)?,
        })
    }
}
