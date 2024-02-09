use super::{Arg, Func, FuncKind};
use crate::{Attribute, Type, expr::Expr};
use sodigy_ast::{GenericDef, IdentWithSpan};
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
};
use sodigy_uid::Uid;

impl Endec for Func {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buf, session);
        self.args.encode(buf, session);
        self.generics.encode(buf, session);
        self.return_val.encode(buf, session);
        self.return_ty.encode(buf, session);
        self.attributes.encode(buf, session);
        self.kind.encode(buf, session);
        self.uid.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Func {
            name: IdentWithSpan::decode(buf, index, session)?,
            args: Option::<Vec<Arg>>::decode(buf, index, session)?,
            generics: Vec::<GenericDef>::decode(buf, index, session)?,
            return_val: Expr::decode(buf, index, session)?,
            return_ty: Option::<Type>::decode(buf, index, session)?,
            attributes: Vec::<Attribute>::decode(buf, index, session)?,
            kind: FuncKind::decode(buf, index, session)?,
            uid: Uid::decode(buf, index, session)?,
        })
    }
}

impl Endec for Arg {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buf, session);
        self.ty.encode(buf, session);
        self.has_question_mark.encode(buf, session);
        self.attributes.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Arg {
            name: IdentWithSpan::decode(buf, index, session)?,
            ty: Option::<Type>::decode(buf, index, session)?,
            has_question_mark: bool::decode(buf, index, session)?,
            attributes: Vec::<Attribute>::decode(buf, index, session)?,
        })
    }
}

impl Endec for FuncKind {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            FuncKind::Normal => { buf.push(0); },
            FuncKind::Lambda => { buf.push(1); },
            FuncKind::Enum { variants } => {
                buf.push(2);
                variants.encode(buf, session);
            },
            FuncKind::EnumVariant { parent } => {
                buf.push(3);
                parent.encode(buf, session);
            },
            FuncKind::StructConstr => { buf.push(4); },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(FuncKind::Normal),
                    1 => Ok(FuncKind::Lambda),
                    2 => Ok(FuncKind::Enum {
                        variants: Vec::<Uid>::decode(buf, index, session)?,
                    }),
                    3 => Ok(FuncKind::EnumVariant {
                        parent: Uid::decode(buf, index, session)?,
                    }),
                    4 => Ok(FuncKind::StructConstr),
                    5.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl DumpJson for Func {
    fn dump_json(&self) -> JsonObj {
        todo!()
    }
}
