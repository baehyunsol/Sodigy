use super::{ParseWarning, ParseWarningKind};
use smallvec::SmallVec;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_error::ExtraErrInfo;
use sodigy_span::SpanRange;

impl Endec for ParseWarning {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buf, session);
        self.spans.encode(buf, session);
        self.extra.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(ParseWarning {
            kind: ParseWarningKind::decode(buf, index, session)?,
            spans: SmallVec::<[SpanRange; 1]>::decode(buf, index, session)?,
            extra: ExtraErrInfo::decode(buf, index, session)?,
        })
    }
}

impl Endec for ParseWarningKind {
    fn encode(&self, buf: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            ParseWarningKind::NothingToEvalInFString => { buf.push(0); },
            ParseWarningKind::UnmatchedCurlyBrace => { buf.push(1); },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
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
