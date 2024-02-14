use super::ParseSession;
use crate::{ParseError, ParseWarning, TokenTree};
use log::info;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
};
use sodigy_error::UniversalError;
use sodigy_intern::{InternedString, InternSession};
use sodigy_session::{SessionDependency, SessionSnapshot};
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
        self.dependencies.encode(buffer, session);
        self.previous_errors.encode(buffer, session);
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
            dependencies: Vec::<SessionDependency>::decode(buffer, index, session)?,
            previous_errors: Vec::<UniversalError>::decode(buffer, index, session)?,
        })
    }
}

impl DumpJson for ParseSession {
    fn dump_json(&self) -> JsonObj {
        info!("ParseSession::dump_json()");
        todo!()
    }
}
