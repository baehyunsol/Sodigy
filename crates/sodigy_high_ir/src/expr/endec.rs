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
use sodigy_endec::{Endec, EndecErr, EndecSession};
use sodigy_intern::{InternedNumeric, InternedString};
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

impl Endec for Expr {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buf, session);
        self.span.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok(Expr {
            kind: ExprKind::decode(buf, ind, session)?,
            span: SpanRange::decode(buf, ind, session)?,
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
            ExprKind::String { s, is_binary } => {
                buf.push(4);
                s.encode(buf, session);
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
            ExprKind::Match(Match { arms, value }) => {
                buf.push(10);
                arms.encode(buf, session);
                value.encode(buf, session);
            },
            ExprKind::Lambda(Lambda { args, value, captured_values, uid }) => {
                buf.push(11);
                args.encode(buf, session);
                value.encode(buf, session);
                captured_values.encode(buf, session);
                uid.encode(buf, session);
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

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(ExprKind::Identifier(IdentWithOrigin::decode(buf, ind, session)?)),
                    1 => Ok(ExprKind::Integer(InternedNumeric::decode(buf, ind, session)?)),
                    2 => Ok(ExprKind::Ratio(InternedNumeric::decode(buf, ind, session)?)),
                    3 => Ok(ExprKind::Char(char::decode(buf, ind, session)?)),
                    4 => Ok(ExprKind::String {
                        s: InternedString::decode(buf, ind, session)?,
                        is_binary: bool::decode(buf, ind, session)?
                    }),
                    5 => Ok(ExprKind::Call {
                        func: Box::new(Expr::decode(buf, ind, session)?),
                        args: Vec::<Expr>::decode(buf, ind, session)?,
                    }),
                    6 => Ok(ExprKind::List(Vec::<Expr>::decode(buf, ind, session)?)),
                    7 => Ok(ExprKind::Tuple(Vec::<Expr>::decode(buf, ind, session)?)),
                    8 => Ok(ExprKind::Format(Vec::<Expr>::decode(buf, ind, session)?)),
                    9 => Ok(ExprKind::Scope(Scope {
                        original_patterns: Vec::<(Pattern, Expr)>::decode(buf, ind, session)?,
                        lets: Vec::<ScopedLet>::decode(buf, ind, session)?,
                        value: Box::new(Expr::decode(buf, ind, session)?),
                        uid: Uid::decode(buf, ind, session)?,
                    })),
                    10 => Ok(ExprKind::Match(Match {
                        arms: Vec::<MatchArm>::decode(buf, ind, session)?,
                        value: Box::new(Expr::decode(buf, ind, session)?),
                    })),
                    11 => Ok(ExprKind::Lambda(Lambda {
                        args: Vec::<Arg>::decode(buf, ind, session)?,
                        value: Box::new(Expr::decode(buf, ind, session)?),
                        captured_values: Vec::<Expr>::decode(buf, ind, session)?,
                        uid: Uid::decode(buf, ind, session)?,
                    })),
                    12 => Ok(ExprKind::Branch(Branch { arms: Vec::<BranchArm>::decode(buf, ind, session)? })),
                    13 => Ok(ExprKind::StructInit(StructInit {
                        struct_: Box::new(Expr::decode(buf, ind, session)?),
                        fields: Vec::<StructInitField>::decode(buf, ind, session)?,
                    })),
                    14 => Ok(ExprKind::Path {
                        head: Box::new(Expr::decode(buf, ind, session)?),
                        tail: Vec::<IdentWithSpan>::decode(buf, ind, session)?,
                    }),
                    15 => Ok(ExprKind::PrefixOp(
                        PrefixOp::decode(buf, ind, session)?,
                        Box::new(Expr::decode(buf, ind, session)?),
                    )),
                    16 => Ok(ExprKind::PostfixOp(
                        PostfixOp::decode(buf, ind, session)?,
                        Box::new(Expr::decode(buf, ind, session)?),
                    )),
                    17 => Ok(ExprKind::InfixOp(
                        InfixOp::decode(buf, ind, session)?,
                        Box::new(Expr::decode(buf, ind, session)?),
                        Box::new(Expr::decode(buf, ind, session)?),
                    )),
                    18.. => Err(EndecErr::InvalidEnumVariant { variant_index: *n }),
                }
            },
            None => Err(EndecErr::Eof),
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

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok(ScopedLet {
            name: IdentWithSpan::decode(buf, ind, session)?,
            value: Expr::decode(buf, ind, session)?,
            ty: Option::<Type>::decode(buf, ind, session)?,
            is_real: bool::decode(buf, ind, session)?,
        })
    }
}

impl Endec for MatchArm {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.pattern.encode(buf, session);
        self.value.encode(buf, session);
        self.guard.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok(MatchArm {
            pattern: Pattern::decode(buf, ind, session)?,
            value: Expr::decode(buf, ind, session)?,
            guard: Option::<Expr>::decode(buf, ind, session)?,
        })
    }
}

impl Endec for BranchArm {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.cond.encode(buf, session);
        self.pattern_bind.encode(buf, session);
        self.value.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok(BranchArm {
            cond: Option::<Expr>::decode(buf, ind, session)?,
            pattern_bind: Option::<Expr>::decode(buf, ind, session)?,
            value: Expr::decode(buf, ind, session)?,
        })
    }
}

impl Endec for StructInitField {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buf, session);
        self.value.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok(StructInitField {
            name: IdentWithSpan::decode(buf, ind, session)?,
            value: Expr::decode(buf, ind, session)?,
        })
    }
}
