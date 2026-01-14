// If new names are defined (e.g. function params, struct field defs), it checks name collisions.
// If defined names are used (e.g. calling a function with keyword args, initializing a struct), it doesn't check name collisions.

mod attribute;
mod block;
mod r#enum;
mod expr;
mod func;
mod r#if;
mod r#let;
mod r#match;
mod module;
mod pattern;
mod r#struct;
mod r#type;
mod r#use;

pub(crate) use func::check_func_args;
