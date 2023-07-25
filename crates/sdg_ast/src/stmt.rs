mod arg_def;
mod decorator;
mod func_def;
mod mod_def;
mod parse;

#[cfg(test)]
mod tests;

mod use_;

pub use arg_def::{parse_arg_def, ArgDef, GetNameOfArg};
pub use decorator::Decorator;
pub use func_def::{FuncDef, FuncKind};
pub use mod_def::ModDef;
pub use parse::parse_stmts;
pub use use_::{Use, use_case_to_tokens};

#[cfg(test)]
pub use parse::parse_use;

pub enum Stmt {
    // 'def' NAME ('(' ARGS ')')? ':' TYPE '=' EXPR ';'
    Def(FuncDef),

    // has many aliases
    // 'use' PATH 'as' NAME ';'
    Use(Use),

    // '@' DECORATOR_NAME ('(' DECORATOR_ARGS ')')?
    Decorator(Decorator),

    // 'module' MODULE_NAME ';'
    Module(ModDef),
}
