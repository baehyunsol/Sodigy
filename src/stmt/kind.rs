use super::{Decorator, FuncDef, Use};
use crate::module::ModulePath;
use crate::session::InternedString;

pub enum StmtKind {
    // 'def' NAME ('(' ARGS ')')? ':' TYPE '=' EXPR ';'
    Def(FuncDef),

    // has many aliases
    // 'use' PATH 'as' NAME ';'
    Use(Use),

    // '@' DECORATOR_NAME ('(' DECORATOR_ARGS ')')?
    Decorator(Decorator),
}
