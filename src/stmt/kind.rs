use super::{Decorator, FuncDef};

pub enum StmtKind {
    // 'def' NAME ('(' ARGS ')')? ':' TYPE '=' EXPR ';'
    Def(FuncDef),

    // has many aliases
    // 'use' PATH 'as' NAME ';'
    Use,

    // '@' DECORATOR_NAME ('(' DECORATOR_ARGS ')')?
    Decorator(Decorator),
}
