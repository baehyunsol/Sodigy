use crate::Type;
use super::{NumberLike, Pattern, PatternKind, RangeType, StringPattern};
use sodigy_ast::IdentWithSpan;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_intern::{InternedNumeric, InternedString};
use sodigy_span::SpanRange;

impl Endec for Pattern {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buffer, session);
        self.span.encode(buffer, session);
        self.ty.encode(buffer, session);
        self.bind.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Pattern {
            kind: PatternKind::decode(buffer, index, session)?,
            span: SpanRange::decode(buffer, index, session)?,
            ty: Option::<Type>::decode(buffer, index, session)?,
            bind: Option::<IdentWithSpan>::decode(buffer, index, session)?,
        })
    }
}

impl Endec for PatternKind {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            PatternKind::Binding(id) => {
                buffer.push(0);
                id.encode(buffer, session);
            },
            PatternKind::String(st) => {
                buffer.push(1);
                st.encode(buffer, session);
            },
            PatternKind::Range { ty, from, to } => {
                buffer.push(2);
                ty.encode(buffer, session);
                from.encode(buffer, session);
                to.encode(buffer, session);
            },
            PatternKind::Tuple(patterns) => {
                buffer.push(3);
                patterns.encode(buffer, session);
            },
            PatternKind::TupleStruct { name, fields } => {
                buffer.push(4);
                name.encode(buffer, session);
                fields.encode(buffer, session);
            },
            PatternKind::Wildcard => {
                buffer.push(5);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(PatternKind::Binding(InternedString::decode(buffer, index, session)?)),
                    1 => Ok(PatternKind::String(StringPattern::decode(buffer, index, session)?)),
                    2 => Ok(PatternKind::Range {
                        ty: RangeType::decode(buffer, index, session)?,
                        from: NumberLike::decode(buffer, index, session)?,
                        to: NumberLike::decode(buffer, index, session)?,
                    }),
                    3 => Ok(PatternKind::Tuple(Vec::<Pattern>::decode(buffer, index, session)?)),
                    4 => Ok(PatternKind::TupleStruct {
                        name: Vec::<IdentWithSpan>::decode(buffer, index, session)?,
                        fields: Vec::<Pattern>::decode(buffer, index, session)?,
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
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            NumberLike::OpenEnd { is_negative } => {
                buffer.push(0);
                is_negative.encode(buffer, session);
            },
            NumberLike::Exact { num, is_negative } => {
                buffer.push(1);
                num.encode(buffer, session);
                is_negative.encode(buffer, session);
            },
            NumberLike::MinusEpsilon { num, is_negative } => {
                buffer.push(2);
                num.encode(buffer, session);
                is_negative.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(NumberLike::OpenEnd { is_negative: bool::decode(buffer, index, session)? }),
                    1 => Ok(NumberLike::Exact {
                        num: InternedNumeric::decode(buffer, index, session)?,
                        is_negative: bool::decode(buffer, index, session)?,
                    }),
                    2 => Ok(NumberLike::MinusEpsilon {
                        num: InternedNumeric::decode(buffer, index, session)?,
                        is_negative: bool::decode(buffer, index, session)?,
                    }),
                    3.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for RangeType {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            RangeType::Integer => { buffer.push(0); },
            RangeType::Char => { buffer.push(1); },
            RangeType::Ratio => { buffer.push(2); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
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
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.strings.encode(buffer, session);
        self.open_prefix.encode(buffer, session);
        self.open_suffix.encode(buffer, session);
        self.is_binary.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(StringPattern {
            strings: Vec::<IdentWithSpan>::decode(buffer, index, session)?,
            open_prefix: bool::decode(buffer, index, session)?,
            open_suffix: bool::decode(buffer, index, session)?,
            is_binary: bool::decode(buffer, index, session)?,
        })
    }
}
