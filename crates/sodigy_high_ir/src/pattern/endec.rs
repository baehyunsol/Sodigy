use crate::Type;
use super::{NumberLike, Pattern, PatternKind, RangeType};
use sodigy_ast::IdentWithSpan;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_intern::InternedNumeric;
use sodigy_span::SpanRange;

impl Endec for Pattern {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buf, session);
        self.span.encode(buf, session);
        self.ty.encode(buf, session);
        self.bind.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Pattern {
            kind: PatternKind::decode(buf, index, session)?,
            span: SpanRange::decode(buf, index, session)?,
            ty: Option::<Type>::decode(buf, index, session)?,
            bind: Option::<IdentWithSpan>::decode(buf, index, session)?,
        })
    }
}

impl Endec for PatternKind {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        todo!()
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        todo!()
    }
}

impl Endec for NumberLike {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            NumberLike::OpenEnd { is_negative } => {
                buf.push(0);
                is_negative.encode(buf, session);
            },
            NumberLike::Exact { num, is_negative } => {
                buf.push(1);
                num.encode(buf, session);
                is_negative.encode(buf, session);
            },
            NumberLike::MinusEpsilon { num, is_negative } => {
                buf.push(2);
                num.encode(buf, session);
                is_negative.encode(buf, session);
            },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(NumberLike::OpenEnd { is_negative: bool::decode(buf, index, session)? }),
                    1 => Ok(NumberLike::Exact {
                        num: InternedNumeric::decode(buf, index, session)?,
                        is_negative: bool::decode(buf, index, session)?,
                    }),
                    2 => Ok(NumberLike::MinusEpsilon {
                        num: InternedNumeric::decode(buf, index, session)?,
                        is_negative: bool::decode(buf, index, session)?,
                    }),
                    3.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for RangeType {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            RangeType::Integer => { buf.push(0); },
            RangeType::Char => { buf.push(1); },
            RangeType::Ratio => { buf.push(2); },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(RangeType::Integer),
                    1 => Ok(RangeType::Char),
                    2 => Ok(RangeType::Ratio),
                    3.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
