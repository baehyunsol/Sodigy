use super::ArgDef;
use crate::expr::Expr;
use crate::session::InternedString;

pub struct FuncDef {
    pub name: InternedString,
    pub args: Vec<ArgDef>,

    pub ret_type: Expr,
    pub ret_val: Expr,

    // constants are defined without args 
    // 0-arg functions and constants are different: `def PI` vs `def GET_PI()`
    pub is_const: bool,
}
