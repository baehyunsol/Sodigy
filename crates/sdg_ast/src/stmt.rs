mod arg_def;
mod enum_def;
mod decorator;
mod func_def;
mod generic_def;
mod mod_def;
mod parse;

#[cfg(test)]
mod tests;

mod use_;
mod variant_def;

pub use arg_def::{parse_arg_def, ArgDef};
pub use enum_def::EnumDef;
pub use decorator::Decorator;
pub use func_def::{FuncDef, FuncKind, LAMBDA_FUNC_PREFIX};
pub use generic_def::GenericDef;
pub use mod_def::ModDef;
pub use parse::parse_stmts;
pub use use_::{Use, use_case_to_tokens};
pub use variant_def::VariantDef;

#[cfg(test)]
pub use parse::parse_use;

use crate::session::InternedString;
use crate::value::BlockDef;

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

    // 'enum' ENUM_NAME ';'
    // 'enum' ENUM_NAME '{' (ENUM_VAR ',')* '}'
    Enum(EnumDef),
}

pub trait GetNameOfArg {
    fn get_name_of_arg(&self) -> InternedString;
}

impl GetNameOfArg for InternedString {
    fn get_name_of_arg(&self) -> InternedString {
        *self
    }
}

impl<T> GetNameOfArg for (InternedString, T) {
    fn get_name_of_arg(&self) -> InternedString {
        self.0
    }
}

impl GetNameOfArg for ArgDef {
    fn get_name_of_arg(&self) -> InternedString {
        self.name
    }
}

impl GetNameOfArg for BlockDef {
    fn get_name_of_arg(&self) -> InternedString {
        self.name
    }
}

impl GetNameOfArg for GenericDef {
    fn get_name_of_arg(&self) -> InternedString {
        self.name
    }
}

impl<A: GetNameOfArg> GetNameOfArg for Box<A> {
    fn get_name_of_arg(&self) -> InternedString {
        self.as_ref().get_name_of_arg()
    }
}
