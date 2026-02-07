use crate::{Expr, PatternKind, Session};

impl PatternKind {
    pub fn from_expr(expr: &Expr, session: &mut Session) -> Result<PatternKind, ()> {
        match expr {
            Expr::Constant(c) => Ok(PatternKind::Constant(c.clone())),
            _ => todo!(),
        }
    }
}
