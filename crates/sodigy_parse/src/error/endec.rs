use super::{ParseError, ParseErrorKind};
use crate::token_tree::TokenTreeKind;
use smallvec::SmallVec;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_error::{ExpectedToken, ExtraErrInfo};
use sodigy_span::SpanRange;

impl Endec for ParseError {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buffer, session);
        self.spans.encode(buffer, session);
        self.extra.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(ParseError {
            kind: ParseErrorKind::decode(buffer, index, session)?,
            spans: SmallVec::<[SpanRange; 1]>::decode(buffer, index, session)?,
            extra: ExtraErrInfo::decode(buffer, index, session)?,
        })
    }
}

impl Endec for ParseErrorKind {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            ParseErrorKind::UnfinishedDelim(c) => {
                buffer.push(0);
                c.encode(buffer, session);
            },
            ParseErrorKind::MismatchDelim(c) => {
                buffer.push(1);
                c.encode(buffer, session);
            },
            ParseErrorKind::EmptyFString => { buffer.push(2); },
            ParseErrorKind::FStringSingleQuote => { buffer.push(3); },
            ParseErrorKind::FStringWithoutPrefix {
                has_prefix_b
            } => {
                buffer.push(4);
                has_prefix_b.encode(buffer, session);
            },
            ParseErrorKind::ThreeDots => { buffer.push(5); },
            ParseErrorKind::LonelyBacktick => { buffer.push(6); },
            ParseErrorKind::LonelyBackslash => { buffer.push(7); },
            ParseErrorKind::UnexpectedToken(kind, expected) => {
                buffer.push(8);
                kind.encode(buffer, session);
                expected.encode(buffer, session);
            },
            ParseErrorKind::UnexpectedEof(expected) => {
                buffer.push(9);
                expected.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(ParseErrorKind::UnfinishedDelim(u8::decode(buffer, index, session)?)),
                    1 => Ok(ParseErrorKind::MismatchDelim(u8::decode(buffer, index, session)?)),
                    2 => Ok(ParseErrorKind::EmptyFString),
                    3 => Ok(ParseErrorKind::FStringSingleQuote),
                    4 => Ok(ParseErrorKind::FStringWithoutPrefix {
                        has_prefix_b: bool::decode(buffer, index, session)?,
                    }),
                    5 => Ok(ParseErrorKind::ThreeDots),
                    6 => Ok(ParseErrorKind::LonelyBacktick),
                    7 => Ok(ParseErrorKind::LonelyBackslash),
                    8 => Ok(ParseErrorKind::UnexpectedToken(
                        TokenTreeKind::decode(buffer, index, session)?,
                        ExpectedToken::<TokenTreeKind>::decode(buffer, index, session)?,
                    )),
                    9 => Ok(ParseErrorKind::UnexpectedEof(
                        ExpectedToken::<TokenTreeKind>::decode(buffer, index, session)?,
                    )),
                    10.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
