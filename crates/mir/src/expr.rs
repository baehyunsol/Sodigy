use crate::{Block, If};
use sodigy_hir::{self as hir, IdentWithOrigin};
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
        tail_call: bool,
    },
}

pub enum Callable {
    // There must be `HashMap<Span, Func>` somewhere
    Static(Span),
}

pub fn from_hir(hir_expr: &hir::Expr) -> Result<Expr, ()> {
    match hir_expr {
        hir::Expr::Identifier(id) => Ok(Expr::Identifier(*id)),
        hir::Expr::Number { n, span } => Ok(Expr::Number {
            n: *n,
            span: *span,
        }),
        hir::Expr::If(r#if) => todo!(),
        hir::Expr::Block(block) => match Block::from_hir(block) {
            Ok(block) => Ok(Expr::Block(block)),
            Err(_) => Err(()),
        },
        hir::Expr::Call {
            func,
            args,
        } => todo!(),

        // TODO: it has to be `mir::Expr::Call`, but how?
        hir::Expr::InfixOp { op, lhs, rhs } => todo!(),
    }
}
