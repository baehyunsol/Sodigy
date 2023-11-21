use super::{Expr, ExprKind, LocalDef, Match, MatchArm, Scope};
use crate::pattern::Pattern;
use sodigy_ast::IdentWithSpan;
use sodigy_endec::{Endec, EndecErr, EndecSession};
use sodigy_span::SpanRange;

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
                defs,
                value,
                uid,
            }) => {
                buf.push(9);
                original_patterns.encode(buf, session);
                defs.encode(buf, session);
                value.encode(buf, session);
                uid.encode(buf, session);
            },
            ExprKind::Match(Match { arms, value }) => {
                buf.push(10);
                arms.encode(buf, session);
                value.encode(buf, session);
            },
            _ => todo!(),
        }
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    _ => todo!(),
                    n => Err(EndecErr::InvalidEnumVariant { variant_index: n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}

impl Endec for LocalDef {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.name.encode(buf, session);
        self.value.encode(buf, session);
        self.is_real.encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok(LocalDef {
            name: IdentWithSpan::decode(buf, ind, session)?,
            value: Expr::decode(buf, ind, session)?,
            is_real: bool::decode(buf, ind, session)?,
        })
    }
}

/*

    pub pattern: Pattern,
    pub value: Expr,
    pub guard: Option<Expr>,
*/
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
