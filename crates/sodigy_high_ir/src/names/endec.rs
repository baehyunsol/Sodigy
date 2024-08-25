use super::{IdentWithOrigin, NameBindingType, NameOrigin};
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
    json_key_value_table,
};
use sodigy_intern::InternedString;
use sodigy_uid::Uid;

impl Endec for IdentWithOrigin {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.0.encode(buffer, session);
        self.1.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(IdentWithOrigin(
            InternedString::decode(buffer, index, session)?,
            NameOrigin::decode(buffer, index, session)?,
        ))
    }
}

impl Endec for NameOrigin {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            NameOrigin::Prelude => {
                buffer.push(0);
            },
            NameOrigin::LangItem => {
                buffer.push(1);
            },
            NameOrigin::FuncArg { index } => {
                buffer.push(2);
                index.encode(buffer, session);
            },
            NameOrigin::FuncGeneric { index } => {
                buffer.push(3);
                index.encode(buffer, session);
            },
            NameOrigin::Local { origin } => {
                buffer.push(4);
                origin.encode(buffer, session);
            },
            NameOrigin::Global { origin } => {
                buffer.push(5);
                origin.encode(buffer, session);
            },
            NameOrigin::Captured { lambda, index } => {
                buffer.push(6);
                lambda.encode(buffer, session);
                index.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(NameOrigin::Prelude),
                    1 => Ok(NameOrigin::LangItem),
                    2 => Ok(NameOrigin::FuncArg { index: usize::decode(buffer, index, session)? }),
                    3 => Ok(NameOrigin::FuncGeneric { index: usize::decode(buffer, index, session)? }),
                    4 => Ok(NameOrigin::Local { origin: Uid::decode(buffer, index, session)? }),
                    5 => Ok(NameOrigin::Global { origin: Option::<Uid>::decode(buffer, index, session)? }),
                    6 => Ok(NameOrigin::Captured {
                        lambda: Uid::decode(buffer, index, session)?,
                        index: usize::decode(buffer, index, session)?,
                    }),
                    7.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for NameBindingType {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            NameBindingType::ScopedLet => { buffer.push(0); },
            NameBindingType::FuncArg => { buffer.push(1); },
            NameBindingType::FuncGeneric => { buffer.push(2); },
            NameBindingType::LambdaArg => { buffer.push(3); },
            NameBindingType::MatchArm => { buffer.push(4); },
            NameBindingType::IfPattern => { buffer.push(5); },
            NameBindingType::Import => { buffer.push(6); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(NameBindingType::ScopedLet),
                    1 => Ok(NameBindingType::FuncArg),
                    2 => Ok(NameBindingType::FuncGeneric),
                    3 => Ok(NameBindingType::LambdaArg),
                    4 => Ok(NameBindingType::MatchArm),
                    5 => Ok(NameBindingType::IfPattern),
                    6 => Ok(NameBindingType::Import),
                    7.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl DumpJson for IdentWithOrigin {
    fn dump_json(&self) -> JsonObj {
        json_key_value_table(vec![
            ("identifier", self.0.dump_json()),
            ("origin", self.1.dump_json()),
        ])
    }
}

impl DumpJson for NameOrigin {
    fn dump_json(&self) -> JsonObj {
        match self {
            NameOrigin::Prelude => json_key_value_table(vec![("type", "prelude".dump_json())]),
            NameOrigin::LangItem => json_key_value_table(vec![("type", "lang_item".dump_json())]),
            NameOrigin::FuncArg { index } => json_key_value_table(vec![
                ("type", "func_arg".dump_json()),
                ("index", index.dump_json()),
            ]),
            NameOrigin::FuncGeneric { index } => json_key_value_table(vec![
                ("type", "func_generic".dump_json()),
                ("index", index.dump_json()),
            ]),
            NameOrigin::Local { origin } => json_key_value_table(vec![
                ("type", "local".dump_json()),
                ("origin", origin.dump_json()),
            ]),
            NameOrigin::Global { origin } => json_key_value_table(vec![
                ("type", "global".dump_json()),
                ("origin", origin.dump_json()),
            ]),
            NameOrigin::Captured { lambda, index } => json_key_value_table(vec![
                ("type", "captured".dump_json()),
                ("lambda", lambda.dump_json()),
                ("index", index.dump_json()),
            ]),
        }
    }
}
