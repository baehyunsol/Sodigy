use crate::Type;
use super::{NumberLike, Pattern, PatternKind, RangeType, StringPattern};
use sodigy_ast::IdentWithSpan;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
    json_key_value_table,
};
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
            NumberLike::Exact(num) => {
                buffer.push(1);
                num.encode(buffer, session);
            },
            NumberLike::MinusEpsilon(num) => {
                buffer.push(2);
                num.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(NumberLike::OpenEnd { is_negative: bool::decode(buffer, index, session)? }),
                    1 => Ok(NumberLike::Exact(InternedNumeric::decode(buffer, index, session)?)),
                    2 => Ok(NumberLike::MinusEpsilon(InternedNumeric::decode(buffer, index, session)?)),
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

impl DumpJson for Pattern {
    fn dump_json(&self) -> JsonObj {
        json_key_value_table(vec![
            ("kind", self.kind.dump_json()),
            ("span", self.span.dump_json()),
            ("ty", self.ty.dump_json()),
            ("bind", self.bind.dump_json()),
        ])
    }
}

impl DumpJson for PatternKind {
    fn dump_json(&self) -> JsonObj {
        match self {
            PatternKind::Binding(name) => json_key_value_table(vec![
                ("binding", name.dump_json()),
            ]),
            PatternKind::String(string_pattern) => json_key_value_table(vec![
                ("string_pattern", string_pattern.dump_json()),
            ]),
            PatternKind::Range { ty, from, to } => json_key_value_table(vec![
                ("range", ty.dump_json()),
                ("from", from.dump_json()),
                ("to", to.dump_json()),
            ]),
            PatternKind::Tuple(elements) => json_key_value_table(vec![
                ("tuple", elements.dump_json()),
            ]),
            PatternKind::TupleStruct {
                name,
                fields,
            } => json_key_value_table(vec![
                ("name", name.dump_json()),
                ("fields", fields.dump_json()),
            ]),
            PatternKind::Wildcard => "wildcard".dump_json(),
        }
    }
}

impl DumpJson for StringPattern {
    fn dump_json(&self) -> JsonObj {
        json_key_value_table(vec![
            ("strings", self.strings.dump_json()),
            ("open_prefix", self.open_prefix.dump_json()),
            ("open_suffix", self.open_suffix.dump_json()),
            ("is_binary", self.is_binary.dump_json()),
        ])
    }
}

impl DumpJson for RangeType {
    fn dump_json(&self) -> JsonObj {
        format!("{self:?}").dump_json()
    }
}

impl DumpJson for NumberLike {
    fn dump_json(&self) -> JsonObj {
        match self {
            NumberLike::OpenEnd { is_negative } => format!(
                "{} infinite",
                if *is_negative { "negative" } else { "positive" },
            ),
            NumberLike::Exact(n) => n.to_string(),
            NumberLike::MinusEpsilon(n) => format!("{n} - epsilon"),
        }.dump_json()
    }
}
