use super::{Arg, Func, FuncKind};
use crate::{Attribute, Type, expr::Expr};
use sodigy_ast::GenericDef;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
    json_key_value_table,
};
use sodigy_parse::IdentWithSpan;
use sodigy_uid::Uid;

impl Endec for Func {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buffer, session);
        self.args.encode(buffer, session);
        self.generics.encode(buffer, session);
        self.return_val.encode(buffer, session);
        self.return_ty.encode(buffer, session);
        self.attributes.encode(buffer, session);
        self.kind.encode(buffer, session);
        self.uid.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Func {
            name: IdentWithSpan::decode(buffer, index, session)?,
            args: Option::<Vec<Arg>>::decode(buffer, index, session)?,
            generics: Vec::<GenericDef>::decode(buffer, index, session)?,
            return_val: Expr::decode(buffer, index, session)?,
            return_ty: Option::<Type>::decode(buffer, index, session)?,
            attributes: Vec::<Attribute>::decode(buffer, index, session)?,
            kind: FuncKind::decode(buffer, index, session)?,
            uid: Uid::decode(buffer, index, session)?,
        })
    }
}

impl Endec for Arg {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buffer, session);
        self.ty.encode(buffer, session);
        self.has_question_mark.encode(buffer, session);
        self.attributes.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Arg {
            name: IdentWithSpan::decode(buffer, index, session)?,
            ty: Option::<Type>::decode(buffer, index, session)?,
            has_question_mark: bool::decode(buffer, index, session)?,
            attributes: Vec::<Attribute>::decode(buffer, index, session)?,
        })
    }
}

impl Endec for FuncKind {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            FuncKind::Normal => { buffer.push(0); },
            FuncKind::Lambda => { buffer.push(1); },
            FuncKind::Enum { variants } => {
                buffer.push(2);
                variants.encode(buffer, session);
            },
            FuncKind::EnumVariant { parent } => {
                buffer.push(3);
                parent.encode(buffer, session);
            },
            FuncKind::StructDef => { buffer.push(4); },
            FuncKind::StructConstr => { buffer.push(5); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(FuncKind::Normal),
                    1 => Ok(FuncKind::Lambda),
                    2 => Ok(FuncKind::Enum {
                        variants: Vec::<Uid>::decode(buffer, index, session)?,
                    }),
                    3 => Ok(FuncKind::EnumVariant {
                        parent: Uid::decode(buffer, index, session)?,
                    }),
                    4 => Ok(FuncKind::StructDef),
                    5 => Ok(FuncKind::StructConstr),
                    6.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl DumpJson for Func {
    fn dump_json(&self) -> JsonObj {
        json_key_value_table(vec![
            ("name", self.name.dump_json()),
            ("arguments", self.args.dump_json()),
            ("generics", self.generics.dump_json()),
            ("return_value", self.return_val.dump_json()),
            ("return_type_annotation", self.return_ty.dump_json()),
            ("attributes", self.attributes.dump_json()),
            ("uid", self.uid.dump_json()),
            ("kind", self.kind.dump_json()),
            ("rendered", self.to_string().dump_json()),
        ])
    }
}

impl DumpJson for Arg {
    fn dump_json(&self) -> JsonObj {
        json_key_value_table(vec![
            ("name", self.name.dump_json()),
            ("type_annotation", self.ty.dump_json()),
            ("has_question_mark", self.has_question_mark.dump_json()),
            ("attributes", self.attributes.dump_json()),
        ])
    }
}

impl DumpJson for FuncKind {
    fn dump_json(&self) -> JsonObj {
        match self {
            FuncKind::Normal => "normal".dump_json(),
            FuncKind::Lambda => "lambda".dump_json(),
            FuncKind::Enum { variants } => json_key_value_table(vec![
                ("enum", variants.dump_json()),
            ]),
            FuncKind::EnumVariant { parent } => json_key_value_table(vec![
                ("kind", "enum_variant".dump_json()),
                ("parent", parent.dump_json()),
            ]),
            FuncKind::StructDef => "struct_def".dump_json(),
            FuncKind::StructConstr => "struct_constructor".dump_json(),
        }
    }
}
