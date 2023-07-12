use crate::expr::Expr;
use crate::session::InternedString;

pub struct Decorator {
    pub name: InternedString,
    pub args: Vec<Expr>,

    // 0-args and no_args are different
    // `@deco` vs `@deco()`
    pub no_args: bool,
}
