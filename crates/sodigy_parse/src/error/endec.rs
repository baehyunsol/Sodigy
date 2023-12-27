use super::{ParseError, ParseErrorKind};
use crate::token_tree::TokenTreeKind;
use smallvec::SmallVec;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_error::{ExpectedToken, ExtraErrInfo};
use sodigy_span::SpanRange;

impl Endec for ParseError {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buf, session);
        self.spans.encode(buf, session);
        self.extra.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(ParseError {
            kind: ParseErrorKind::decode(buf, index, session)?,
            spans: SmallVec::<[SpanRange; 1]>::decode(buf, index, session)?,
            extra: ExtraErrInfo::decode(buf, index, session)?,
        })
    }
}

impl Endec for ParseErrorKind {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            ParseErrorKind::UnfinishedDelim(c) => {
                buf.push(0);
                c.encode(buf, session);
            },
            ParseErrorKind::MismatchDelim(c) => {
                buf.push(1);
                c.encode(buf, session);
            },
            ParseErrorKind::EmptyFString => { buf.push(2); },
            ParseErrorKind::FStringSingleQuote => { buf.push(3); },
            ParseErrorKind::FStringWithoutPrefix {
                has_prefix_b
            } => {
                buf.push(4);
                has_prefix_b.encode(buf, session);
            },
            ParseErrorKind::ThreeDots => { buf.push(5); },
            ParseErrorKind::LonelyBacktick => { buf.push(6); },
            ParseErrorKind::LonelyBackslash => { buf.push(7); },
            ParseErrorKind::UnexpectedToken(kind, expected) => {
                buf.push(8);
                kind.encode(buf, session);
                expected.encode(buf, session);
            },
            ParseErrorKind::NumericExpOverflow => { buf.push(9); },
            ParseErrorKind::TODO(s) => {
                buf.push(10);
                s.encode(buf, session);
            },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(ParseErrorKind::UnfinishedDelim(u8::decode(buf, index, session)?)),
                    1 => Ok(ParseErrorKind::MismatchDelim(u8::decode(buf, index, session)?)),
                    2 => Ok(ParseErrorKind::EmptyFString),
                    3 => Ok(ParseErrorKind::FStringSingleQuote),
                    4 => Ok(ParseErrorKind::FStringWithoutPrefix {
                        has_prefix_b: bool::decode(buf, index, session)?,
                    }),
                    5 => Ok(ParseErrorKind::ThreeDots),
                    6 => Ok(ParseErrorKind::LonelyBacktick),
                    7 => Ok(ParseErrorKind::LonelyBackslash),
                    8 => Ok(ParseErrorKind::UnexpectedToken(
                        TokenTreeKind::decode(buf, index, session)?,
                        ExpectedToken::<TokenTreeKind>::decode(buf, index, session)?,
                    )),
                    9 => Ok(ParseErrorKind::NumericExpOverflow),
                    10 => Ok(ParseErrorKind::TODO(String::decode(buf, index, session)?)),
                    11.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
