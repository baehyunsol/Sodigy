use super::{ParseWarning, ParseWarningKind};
use smallvec::SmallVec;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_error::ExtraErrInfo;
use sodigy_span::SpanRange;

impl Endec for ParseWarning {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buffer, session);
        self.spans.encode(buffer, session);
        self.extra.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(ParseWarning {
            kind: ParseWarningKind::decode(buffer, index, session)?,
            spans: SmallVec::<[SpanRange; 1]>::decode(buffer, index, session)?,
            extra: ExtraErrInfo::decode(buffer, index, session)?,
        })
    }
}

impl Endec for ParseWarningKind {
    fn encode(&self, buffer: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            ParseWarningKind::NothingToEvalInFString => { buffer.push(0); },
            ParseWarningKind::UnmatchedCurlyBrace => { buffer.push(1); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(ParseWarningKind::NothingToEvalInFString),
                    1 => Ok(ParseWarningKind::UnmatchedCurlyBrace),
                    2.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
