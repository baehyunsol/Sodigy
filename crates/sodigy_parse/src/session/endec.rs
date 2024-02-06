use super::ParseSession;
use crate::{ParseError, ParseWarning, TokenTree};
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_error::UniversalError;
use sodigy_intern::{InternedString, InternSession};
use sodigy_session::{SessionDependency, SessionSnapshot};
use std::collections::HashSet;

impl Endec for ParseSession {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        // there's no point in encoding InternSession

        self.tokens.encode(buf, session);
        self.errors.encode(buf, session);
        self.warnings.encode(buf, session);
        self.unexpanded_macros.encode(buf, session);
        self.snapshots.encode(buf, session);
        self.dependencies.encode(buf, session);
        self.previous_errors.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        // there's no point in decoding InternSession

        Ok(ParseSession {
            tokens: Vec::<TokenTree>::decode(buf, index, session)?,
            errors: Vec::<ParseError>::decode(buf, index, session)?,
            warnings: Vec::<ParseWarning>::decode(buf, index, session)?,
            interner: InternSession::new(),
            unexpanded_macros: HashSet::<InternedString>::decode(buf, index, session)?,
            snapshots: Vec::<SessionSnapshot>::decode(buf, index, session)?,
            dependencies: Vec::<SessionDependency>::decode(buf, index, session)?,
            previous_errors: Vec::<UniversalError>::decode(buf, index, session)?,
        })
    }
}
