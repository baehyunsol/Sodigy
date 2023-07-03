use crate::expr::Expr;
use crate::session::InternedString;

pub struct Decorator {
    name: InternedString,
    args: Vec<Expr>,
}
