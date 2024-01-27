use crate::Type;
use super::{NumberLike, Pattern, PatternKind, RangeType, StringPattern};
use sodigy_ast::IdentWithSpan;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_intern::{InternedNumeric, InternedString};
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
        match self {
            PatternKind::Binding(id) => {
                buf.push(0);
                id.encode(buf, session);
            },
            PatternKind::String(st) => {
                buf.push(1);
                st.encode(buf, session);
            },
            PatternKind::Range { ty, from, to } => {
                buf.push(2);
                ty.encode(buf, session);
                from.encode(buf, session);
                to.encode(buf, session);
            },
            PatternKind::Tuple(patterns) => {
                buf.push(3);
                patterns.encode(buf, session);
            },
            PatternKind::TupleStruct { name, fields } => {
                buf.push(4);
                name.encode(buf, session);
                fields.encode(buf, session);
            },
            PatternKind::Wildcard => {
                buf.push(5);
            },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(PatternKind::Binding(InternedString::decode(buf, index, session)?)),
                    1 => Ok(PatternKind::String(StringPattern::decode(buf, index, session)?)),
                    2 => Ok(PatternKind::Range {
                        ty: RangeType::decode(buf, index, session)?,
                        from: NumberLike::decode(buf, index, session)?,
                        to: NumberLike::decode(buf, index, session)?,
                    }),
                    3 => Ok(PatternKind::Tuple(Vec::<Pattern>::decode(buf, index, session)?)),
                    4 => Ok(PatternKind::TupleStruct {
                        name: Vec::<IdentWithSpan>::decode(buf, index, session)?,
                        fields: Vec::<Pattern>::decode(buf, index, session)?,
                    }),
                    5 => Ok(PatternKind::Wildcard),
                    6.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
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

impl Endec for StringPattern {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.strings.encode(buf, session);
        self.open_prefix.encode(buf, session);
        self.open_suffix.encode(buf, session);
        self.is_binary.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(StringPattern {
            strings: Vec::<IdentWithSpan>::decode(buf, index, session)?,
            open_prefix: bool::decode(buf, index, session)?,
            open_suffix: bool::decode(buf, index, session)?,
            is_binary: bool::decode(buf, index, session)?,
        })
    }
}
