use super::ArgDef;
use crate::expr::Expr;
use crate::session::InternedString;

pub struct FuncDef {
    pub name: InternedString,
    pub args: Vec<ArgDef>,

    pub ret_type: Expr,
    pub ret_val: Expr,

    // constants are defined without args (but 0-arg functions and constants are different)
    // constants cannot be called
    pub is_const: bool,
}
