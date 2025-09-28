use crate::{Block, If, Session};
use sodigy_hir as hir;
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_number::InternedNumber;
use sodigy_span::Span;

pub enum Expr {
    Identifier(IdentWithOrigin),
    Number {
        n: InternedNumber,
        span: Span,
    },
    If(If),
    Block(Block),
    Call {
        func: Callable,
        args: Vec<Expr>,

        // We have to do tail-call analysis after function inlining!
        // tail_call: bool,
    },
}

pub enum Callable {
    // There must be `HashMap<Span, Func>` somewhere
    Static(Span),
}

impl Expr {
    pub fn from_hir(hir_expr: &hir::Expr, session: &mut Session) -> Result<Expr, ()> {
        match hir_expr {
            hir::Expr::Identifier(id) => Ok(Expr::Identifier(*id)),
            hir::Expr::Number { n, span } => Ok(Expr::Number {
                n: *n,
                span: *span,
            }),
            hir::Expr::If(r#if) => todo!(),
            hir::Expr::Block(block) => match Block::from_hir(block, session) {
                Ok(block) => Ok(Expr::Block(block)),
                Err(_) => Err(()),
            },
            hir::Expr::Call {
                func,
                args,
            } => {
                let mut has_error = false;

                let func = match Expr::from_hir(func, session) {
                    Ok(Expr::Identifier(id)) => match id.origin {
                        NameOrigin::Local { kind } |
                        NameOrigin::Foreign { kind } => match kind {
                            NameKind::Func => Callable::Static(id.def_span),
                            _ => todo!(),
                        },
                        NameOrigin::FuncArg { .. } => todo!(),
                    },
                    Ok(id) => todo!(),
                    Err(_) => {
                        has_error = true;
                        todo!()
                    },
                };

                todo!()
            },

            // TODO: these have to be `mir::Expr::Call`, but how?
            hir::Expr::Tuple { elements, .. } => todo!(),
            hir::Expr::List { elements, .. } => todo!(),
            hir::Expr::InfixOp { op, lhs, rhs } => todo!(),
        }
    }
}
