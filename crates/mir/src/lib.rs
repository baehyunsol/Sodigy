pub enum Expr {
    Identifier {},
    Number {},
    If(),
    Block(),
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

pub fn from_hir(hir_expr: &hir::Expr) {
    match hir_expr {
        hir::Expr::Identifier {} => {},
        hir::Expr::Number {} => {},
        hir::Expr::If() => {},
        hir::Expr::Block {} => {},
        hir::Expr::Call {
            func,
            args,
        } => {},

        // TODO: it has to be `mir::Expr::Call`, but how?
        hir::Expr::InfixOp {} => {},
    }
}
