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
};
use sodigy_intern::{InternedNumeric, InternedString};
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

impl Endec for Expr {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buf, session);
        self.span.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Expr {
            kind: ExprKind::decode(buf, index, session)?,
            span: SpanRange::decode(buf, index, session)?,
        })
    }
}

impl Endec for ExprKind {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            ExprKind::Identifier(id) => {
                buf.push(0);
                id.encode(buf, session);
            },
            ExprKind::Integer(n) => {
                buf.push(1);
                n.encode(buf, session);
            },
            ExprKind::Ratio(n) => {
                buf.push(2);
                n.encode(buf, session);
            },
            ExprKind::Char(c) => {
                buf.push(3);
                c.encode(buf, session);
            },
            ExprKind::String { content, is_binary } => {
                buf.push(4);
                content.encode(buf, session);
                is_binary.encode(buf, session);
            },
            ExprKind::Call { func, args } => {
                buf.push(5);
                func.encode(buf, session);
                args.encode(buf, session);
            },
            ExprKind::List(elements) => {
                buf.push(6);
                elements.encode(buf, session);
            },
            ExprKind::Tuple(elements) => {
                buf.push(7);
                elements.encode(buf, session);
            },
            ExprKind::Format(elements) => {
                buf.push(8);
                elements.encode(buf, session);
            },
            ExprKind::Scope(Scope {
                original_patterns,
                lets,
                value,
                uid,
            }) => {
                buf.push(9);
                original_patterns.encode(buf, session);
                lets.encode(buf, session);
                value.encode(buf, session);
                uid.encode(buf, session);
            },
            ExprKind::Match(Match { arms, value, is_lowered_from_if_pattern }) => {
                buf.push(10);
                arms.encode(buf, session);
                value.encode(buf, session);
                is_lowered_from_if_pattern.encode(buf, session);
            },
            ExprKind::Lambda(Lambda {
                args,
                value,
                captured_values,
                uid,
                return_ty,
                lowered_from_scoped_let,
            }) => {
                buf.push(11);
                args.encode(buf, session);
                value.encode(buf, session);
                captured_values.encode(buf, session);
                uid.encode(buf, session);
                return_ty.encode(buf, session);
                lowered_from_scoped_let.encode(buf, session);
            },
            ExprKind::Branch(Branch { arms }) => {
                buf.push(12);
                arms.encode(buf, session);
            },
            ExprKind::StructInit(StructInit { struct_, fields }) => {
                buf.push(13);
                struct_.encode(buf, session);
                fields.encode(buf, session);
            },
            ExprKind::Path { head, tail } => {
                buf.push(14);
                head.encode(buf, session);
                tail.encode(buf, session);
            },
            ExprKind::PrefixOp(op, val) => {
                buf.push(15);
                op.encode(buf, session);
                val.encode(buf, session);
            },
            ExprKind::PostfixOp(op, val) => {
                buf.push(16);
                op.encode(buf, session);
                val.encode(buf, session);
            },
            ExprKind::InfixOp(op, lhs, rhs) => {
                buf.push(17);
                op.encode(buf, session);
                lhs.encode(buf, session);
                rhs.encode(buf, session);
            },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(ExprKind::Identifier(IdentWithOrigin::decode(buf, index, session)?)),
                    1 => Ok(ExprKind::Integer(InternedNumeric::decode(buf, index, session)?)),
                    2 => Ok(ExprKind::Ratio(InternedNumeric::decode(buf, index, session)?)),
                    3 => Ok(ExprKind::Char(char::decode(buf, index, session)?)),
                    4 => Ok(ExprKind::String {
                        content: InternedString::decode(buf, index, session)?,
                        is_binary: bool::decode(buf, index, session)?
                    }),
                    5 => Ok(ExprKind::Call {
                        func: Box::new(Expr::decode(buf, index, session)?),
                        args: Vec::<Expr>::decode(buf, index, session)?,
                    }),
                    6 => Ok(ExprKind::List(Vec::<Expr>::decode(buf, index, session)?)),
                    7 => Ok(ExprKind::Tuple(Vec::<Expr>::decode(buf, index, session)?)),
                    8 => Ok(ExprKind::Format(Vec::<Expr>::decode(buf, index, session)?)),
                    9 => Ok(ExprKind::Scope(Scope {
                        original_patterns: Vec::<(Pattern, Expr)>::decode(buf, index, session)?,
                        lets: Vec::<ScopedLet>::decode(buf, index, session)?,
                        value: Box::new(Expr::decode(buf, index, session)?),
                        uid: Uid::decode(buf, index, session)?,
                    })),
                    10 => Ok(ExprKind::Match(Match {
                        arms: Vec::<MatchArm>::decode(buf, index, session)?,
                        value: Box::new(Expr::decode(buf, index, session)?),
                        is_lowered_from_if_pattern: bool::decode(buf, index, session)?,
                    })),
                    11 => Ok(ExprKind::Lambda(Lambda {
                        args: Vec::<Arg>::decode(buf, index, session)?,
                        value: Box::new(Expr::decode(buf, index, session)?),
                        captured_values: Vec::<Expr>::decode(buf, index, session)?,
                        uid: Uid::decode(buf, index, session)?,
                        return_ty: Option::<Box<Type>>::decode(buf, index, session)?,
                        lowered_from_scoped_let: bool::decode(buf, index, session)?,
                    })),
                    12 => Ok(ExprKind::Branch(Branch { arms: Vec::<BranchArm>::decode(buf, index, session)? })),
                    13 => Ok(ExprKind::StructInit(StructInit {
                        struct_: Box::new(Expr::decode(buf, index, session)?),
                        fields: Vec::<StructInitField>::decode(buf, index, session)?,
                    })),
                    14 => Ok(ExprKind::Path {
                        head: Box::new(Expr::decode(buf, index, session)?),
                        tail: Vec::<IdentWithSpan>::decode(buf, index, session)?,
                    }),
                    15 => Ok(ExprKind::PrefixOp(
                        PrefixOp::decode(buf, index, session)?,
                        Box::new(Expr::decode(buf, index, session)?),
                    )),
                    16 => Ok(ExprKind::PostfixOp(
                        PostfixOp::decode(buf, index, session)?,
                        Box::new(Expr::decode(buf, index, session)?),
                    )),
                    17 => Ok(ExprKind::InfixOp(
                        InfixOp::decode(buf, index, session)?,
                        Box::new(Expr::decode(buf, index, session)?),
                        Box::new(Expr::decode(buf, index, session)?),
                    )),
                    18.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for ScopedLet {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buf, session);
        self.value.encode(buf, session);
        self.ty.encode(buf, session);
        self.is_real.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(ScopedLet {
            name: IdentWithSpan::decode(buf, index, session)?,
            value: Expr::decode(buf, index, session)?,
            ty: Option::<Type>::decode(buf, index, session)?,
            is_real: bool::decode(buf, index, session)?,
        })
    }
}

impl Endec for MatchArm {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.pattern.encode(buf, session);
        self.value.encode(buf, session);
        self.guard.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(MatchArm {
            pattern: Pattern::decode(buf, index, session)?,
            value: Expr::decode(buf, index, session)?,
            guard: Option::<Expr>::decode(buf, index, session)?,
        })
    }
}

impl Endec for BranchArm {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.cond.encode(buf, session);
        self.value.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(BranchArm {
            cond: Option::<Expr>::decode(buf, index, session)?,
            value: Expr::decode(buf, index, session)?,
        })
    }
}

impl Endec for StructInitField {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buf, session);
        self.value.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(StructInitField {
            name: IdentWithSpan::decode(buf, index, session)?,
            value: Expr::decode(buf, index, session)?,
        })
    }
}

impl DumpJson for Expr {
    fn dump_json(&self) -> JsonObj {
        todo!()
    }
}
