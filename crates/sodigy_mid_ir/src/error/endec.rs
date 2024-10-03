use super::{MirError, MirErrorKind};
use smallvec::SmallVec;
use sodigy_endec::{
    Endec,
    EndecError,
    EndecSession,
};
use sodigy_error::ExtraErrorInfo;
use sodigy_high_ir::NameBindingType;
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;

impl Endec for MirError {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buffer, session);
        self.spans.encode(buffer, session);
        self.extra.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(MirError {
            kind: MirErrorKind::decode(buffer, index, session)?,
            spans: SmallVec::<[SpanRange; 1]>::decode(buffer, index, session)?,
            extra: ExtraErrorInfo::decode(buffer, index, session)?,
        })
    }
}

impl Endec for MirErrorKind {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            MirErrorKind::RecursiveLocalValue { name, name_binding_type } => {
                buffer.push(0);
                name.encode(buffer, session);
                name_binding_type.encode(buffer, session);
            },
            MirErrorKind::CycleInLocalValues { names } => {
                buffer.push(1);
                names.encode(buffer, session);
            },
            MirErrorKind::MissingFieldsInStructConstructor { names, struct_name } => {
                buffer.push(2);
                names.encode(buffer, session);
                struct_name.encode(buffer, session);
            },
            MirErrorKind::UnknownFieldsInStructConstructor { names, struct_name } => {
                buffer.push(3);
                names.encode(buffer, session);
                struct_name.encode(buffer, session);
            },
            MirErrorKind::NotAStruct { rendered_expr } => {
                buffer.push(4);
                rendered_expr.encode(buffer, session);
            },
            MirErrorKind::TypeError { expected, got } => {
                buffer.push(5);
                expected.encode(buffer, session);
                got.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(MirErrorKind::RecursiveLocalValue {
                        name: InternedString::decode(buffer, index, session)?,
                        name_binding_type: NameBindingType::decode(buffer, index, session)?,
                    }),
                    1 => Ok(MirErrorKind::CycleInLocalValues {
                        names: Vec::<InternedString>::decode(buffer, index, session)?,
                    }),
                    2 => Ok(MirErrorKind::MissingFieldsInStructConstructor {
                        names: Vec::<InternedString>::decode(buffer, index, session)?,
                        struct_name: InternedString::decode(buffer, index, session)?,
                    }),
                    3 => Ok(MirErrorKind::UnknownFieldsInStructConstructor {
                        names: Vec::<InternedString>::decode(buffer, index, session)?,
                        struct_name: InternedString::decode(buffer, index, session)?,
                    }),
                    4 => Ok(MirErrorKind::NotAStruct {
                        rendered_expr: Option::<String>::decode(buffer, index, session)?,
                    }),
                    5 => Ok(MirErrorKind::TypeError {
                        expected: String::decode(buffer, index, session)?,
                        got: String::decode(buffer, index, session)?,
                    }),
                    6.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
