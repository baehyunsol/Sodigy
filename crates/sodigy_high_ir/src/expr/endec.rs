use super::{
    Branch,
    BranchArm,
    Expr,
    ExprKind,
    Lambda,
    Match,
    MatchArm,
    Scope,
    ScopedLet,
    StructInit,
    StructInitField,
};
use crate::Type;
use crate::func::Arg;
use crate::names::IdentWithOrigin;
use crate::pattern::Pattern;
use sodigy_ast::{
    IdentWithSpan,
    InfixOp,
    PostfixOp,
    PrefixOp,
};
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
use sodigy_uid::Uid;

impl Endec for Expr {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buffer, session);
        self.span.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Expr {
            kind: ExprKind::decode(buffer, index, session)?,
            span: SpanRange::decode(buffer, index, session)?,
        })
    }
}

impl Endec for ExprKind {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            ExprKind::Identifier(id) => {
                buffer.push(0);
                id.encode(buffer, session);
            },
            ExprKind::Integer(n) => {
                buffer.push(1);
                n.encode(buffer, session);
            },
            ExprKind::Ratio(n) => {
                buffer.push(2);
                n.encode(buffer, session);
            },
            ExprKind::Char(c) => {
                buffer.push(3);
                c.encode(buffer, session);
            },
            ExprKind::String { content, is_binary } => {
                buffer.push(4);
                content.encode(buffer, session);
                is_binary.encode(buffer, session);
            },
            ExprKind::Call { func, args } => {
                buffer.push(5);
                func.encode(buffer, session);
                args.encode(buffer, session);
            },
            ExprKind::List(elements) => {
                buffer.push(6);
                elements.encode(buffer, session);
            },
            ExprKind::Tuple(elements) => {
                buffer.push(7);
                elements.encode(buffer, session);
            },
            ExprKind::Format(elements) => {
                buffer.push(8);
                elements.encode(buffer, session);
            },
            ExprKind::Scope(Scope {
                original_patterns,
                lets,
                value,
                uid,
            }) => {
                buffer.push(9);
                original_patterns.encode(buffer, session);
                lets.encode(buffer, session);
                value.encode(buffer, session);
                uid.encode(buffer, session);
            },
            ExprKind::Match(Match { arms, value, is_lowered_from_if_pattern }) => {
                buffer.push(10);
                arms.encode(buffer, session);
                value.encode(buffer, session);
                is_lowered_from_if_pattern.encode(buffer, session);
            },
            ExprKind::Lambda(Lambda {
                args,
                value,
                captured_values,
                uid,
                return_ty,
                lowered_from_scoped_let,
            }) => {
                buffer.push(11);
                args.encode(buffer, session);
                value.encode(buffer, session);
                captured_values.encode(buffer, session);
                uid.encode(buffer, session);
                return_ty.encode(buffer, session);
                lowered_from_scoped_let.encode(buffer, session);
            },
            ExprKind::Branch(Branch { arms }) => {
                buffer.push(12);
                arms.encode(buffer, session);
            },
            ExprKind::StructInit(StructInit { struct_, fields }) => {
                buffer.push(13);
                struct_.encode(buffer, session);
                fields.encode(buffer, session);
            },
            ExprKind::Path { head, tail } => {
                buffer.push(14);
                head.encode(buffer, session);
                tail.encode(buffer, session);
            },
            ExprKind::PrefixOp(op, val) => {
                buffer.push(15);
                op.encode(buffer, session);
                val.encode(buffer, session);
            },
            ExprKind::PostfixOp(op, val) => {
                buffer.push(16);
                op.encode(buffer, session);
                val.encode(buffer, session);
            },
            ExprKind::InfixOp(op, lhs, rhs) => {
                buffer.push(17);
                op.encode(buffer, session);
                lhs.encode(buffer, session);
                rhs.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(ExprKind::Identifier(IdentWithOrigin::decode(buffer, index, session)?)),
                    1 => Ok(ExprKind::Integer(InternedNumeric::decode(buffer, index, session)?)),
                    2 => Ok(ExprKind::Ratio(InternedNumeric::decode(buffer, index, session)?)),
                    3 => Ok(ExprKind::Char(char::decode(buffer, index, session)?)),
                    4 => Ok(ExprKind::String {
                        content: InternedString::decode(buffer, index, session)?,
                        is_binary: bool::decode(buffer, index, session)?
                    }),
                    5 => Ok(ExprKind::Call {
                        func: Box::new(Expr::decode(buffer, index, session)?),
                        args: Vec::<Expr>::decode(buffer, index, session)?,
                    }),
                    6 => Ok(ExprKind::List(Vec::<Expr>::decode(buffer, index, session)?)),
                    7 => Ok(ExprKind::Tuple(Vec::<Expr>::decode(buffer, index, session)?)),
                    8 => Ok(ExprKind::Format(Vec::<Expr>::decode(buffer, index, session)?)),
                    9 => Ok(ExprKind::Scope(Scope {
                        original_patterns: Vec::<(Pattern, Expr)>::decode(buffer, index, session)?,
                        lets: Vec::<ScopedLet>::decode(buffer, index, session)?,
                        value: Box::new(Expr::decode(buffer, index, session)?),
                        uid: Uid::decode(buffer, index, session)?,
                    })),
                    10 => Ok(ExprKind::Match(Match {
                        arms: Vec::<MatchArm>::decode(buffer, index, session)?,
                        value: Box::new(Expr::decode(buffer, index, session)?),
                        is_lowered_from_if_pattern: bool::decode(buffer, index, session)?,
                    })),
                    11 => Ok(ExprKind::Lambda(Lambda {
                        args: Vec::<Arg>::decode(buffer, index, session)?,
                        value: Box::new(Expr::decode(buffer, index, session)?),
                        captured_values: Vec::<Expr>::decode(buffer, index, session)?,
                        uid: Uid::decode(buffer, index, session)?,
                        return_ty: Option::<Box<Type>>::decode(buffer, index, session)?,
                        lowered_from_scoped_let: bool::decode(buffer, index, session)?,
                    })),
                    12 => Ok(ExprKind::Branch(Branch { arms: Vec::<BranchArm>::decode(buffer, index, session)? })),
                    13 => Ok(ExprKind::StructInit(StructInit {
                        struct_: Box::new(Expr::decode(buffer, index, session)?),
                        fields: Vec::<StructInitField>::decode(buffer, index, session)?,
                    })),
                    14 => Ok(ExprKind::Path {
                        head: Box::new(Expr::decode(buffer, index, session)?),
                        tail: Vec::<IdentWithSpan>::decode(buffer, index, session)?,
                    }),
                    15 => Ok(ExprKind::PrefixOp(
                        PrefixOp::decode(buffer, index, session)?,
                        Box::new(Expr::decode(buffer, index, session)?),
                    )),
                    16 => Ok(ExprKind::PostfixOp(
                        PostfixOp::decode(buffer, index, session)?,
                        Box::new(Expr::decode(buffer, index, session)?),
                    )),
                    17 => Ok(ExprKind::InfixOp(
                        InfixOp::decode(buffer, index, session)?,
                        Box::new(Expr::decode(buffer, index, session)?),
                        Box::new(Expr::decode(buffer, index, session)?),
                    )),
                    18.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for ScopedLet {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buffer, session);
        self.value.encode(buffer, session);
        self.ty.encode(buffer, session);
        self.is_real.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(ScopedLet {
            name: IdentWithSpan::decode(buffer, index, session)?,
            value: Expr::decode(buffer, index, session)?,
            ty: Option::<Type>::decode(buffer, index, session)?,
            is_real: bool::decode(buffer, index, session)?,
        })
    }
}

impl Endec for MatchArm {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.pattern.encode(buffer, session);
        self.value.encode(buffer, session);
        self.guard.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(MatchArm {
            pattern: Pattern::decode(buffer, index, session)?,
            value: Expr::decode(buffer, index, session)?,
            guard: Option::<Expr>::decode(buffer, index, session)?,
        })
    }
}

impl Endec for BranchArm {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.cond.encode(buffer, session);
        self.value.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(BranchArm {
            cond: Option::<Expr>::decode(buffer, index, session)?,
            value: Expr::decode(buffer, index, session)?,
        })
    }
}

impl Endec for StructInitField {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buffer, session);
        self.value.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(StructInitField {
            name: IdentWithSpan::decode(buffer, index, session)?,
            value: Expr::decode(buffer, index, session)?,
        })
    }
}

impl DumpJson for Expr {
    fn dump_json(&self) -> JsonObj {
        let mut kind = self.kind.dump_json();
        kind.push_pair("span", self.span.dump_json()).unwrap();

        kind
    }
}

impl DumpJson for ExprKind {
    fn dump_json(&self) -> JsonObj {
        match self {
            ExprKind::Identifier(id) => json_key_value_table(vec![("identifier", id.dump_json())]),
            ExprKind::Integer(n) => json_key_value_table(vec![("integer", n.dump_json())]),
            ExprKind::Ratio(n) => json_key_value_table(vec![("ratio", n.dump_json())]),
            ExprKind::Char(c) => json_key_value_table(vec![("char", (*c as u8).dump_json())]),
            ExprKind::String { content, is_binary } => json_key_value_table(vec![
                ("content", content.dump_json()),
                ("is_binary", is_binary.dump_json()),
            ]),
            ExprKind::Call { func, args } => json_key_value_table(vec![
                ("call", func.as_ref().dump_json()),
                ("arguments", args.dump_json()),
            ]),
            e @ (ExprKind::List(elements)
            | ExprKind::Tuple(elements)) => {
                let name = if matches!(e, ExprKind::List(_)) {
                    "list"
                } else {
                    "tuple"
                };

                json_key_value_table(vec![(name, elements.dump_json())])
            },
            _ => todo!(),
        }
    }
}
