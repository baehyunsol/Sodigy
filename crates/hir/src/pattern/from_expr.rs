use crate::{Expr, PatternKind, Session};

impl PatternKind {
    pub fn from_expr(expr: &Expr, session: &mut Session) -> Result<PatternKind, ()> {
        match expr {
            Expr::Number { n, span } => Ok(PatternKind::Number { n: n.clone(), span: *span }),
            Expr::String { binary, s, span } => Ok(PatternKind::String { binary: *binary, s: *s, span: *span }),
            Expr::Char { ch, span } => Ok(PatternKind::Char { ch: *ch, span: *span }),
            Expr::Byte { b, span } => Ok(PatternKind::Byte { b: *b, span: *span }),
            _ => todo!(),
        }
    }
}
