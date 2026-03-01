use sodigy_parse::Session as ParseSession;

// In sodigy_lex and sodigy_parse, the functions return `Result<T, Vec<Error>>`, and errors are handled by `?` operator.
// That means
// 1. The lexer and the parser doesn't generate warnings.
// 2. They usually halt immediately when there's a syntax error.
//
// But from hir, it has to generate warnings and continue after an error so that it can generate as many errors as possible.
// So all the errors and warnings are stored in the session, and the return value doesn't indicate anything about errors (it does, but don't rely on it).
// You first run the entire hir pass, then you have to check `session.errors` and `session.warnings`.

mod alias;
mod assert;
mod assoc;
mod attribute;
mod block;
mod closure;
pub mod dump;
mod endec;
mod eval;
mod r#enum;
mod expr;
mod func;
mod r#if;
mod item_shape;
mod r#let;
mod r#match;
mod module;
mod name;
mod path;
mod pattern;
mod poly;
mod prelude;
mod session;
mod r#struct;
mod r#type;
mod r#use;

// Func vs FuncShape
// Enum vs EnumShape
// Struct vs StructShape
//
// At first, I needed `HashMap<Span, Func>` for every function in the project (not just a module).
// I needed every mir_session to have the map, and I thought it'd be too expensive to use `Func`
// because it contains a function body.
// So I created `FuncShape`, which is a smaller struct that contains necessary information of a function.
// Then, I needed the same on for structs and enums, so I created `StructShape` and `EnumShape`.
//
// But soon, `StructShape` became as big as `Struct`, so `StructShape` became meaningless.
// I then implemented global-context, which efficiently manages the global maps, so
// `StructShape` and `EnumShape` become even more meaningless.
//
// So I tried to replace `HashMap<Span, StructShape>` in the global map with `HashMap<Span, Struct>`,
// but that didn't work because I added `associated_funcs` and `associated_lets` fields to `StructShape`,
// which don't make sense to be added to `Struct`.
//
// So... here we are. `Struct` the result of lowering `ast::Struct`, while `StructShape` is something
// that's used by mir/bytecode sessions globally.
//
// `ItemShape` is a sum type of `StructShape` and `EnumShape`.

pub use alias::Alias;
pub use assert::Assert;
pub use assoc::{AssociatedFunc, AssociatedItem, AssociatedItemKind};
pub use attribute::{
    ArgCount,
    ArgType,
    Attribute,
    AttributeRule,
    AttributeRuleKey,
    DecoratorArg,
    DecoratorRule,
    KeywordArgRule,
    Requirement,
    Visibility,
    generate_decorator_docs,
};
pub(crate) use attribute::get_decorator_error_notes;
pub use block::Block;
pub(crate) use block::BlockSession;
pub use closure::CapturedNames;
pub use r#enum::{Enum, EnumShape, EnumVariant, EnumVariantFields};
pub use eval::eval_const;
pub use expr::{Expr, ExprOrString};
pub use func::{CallArg, Func, FuncOrigin, FuncParam, FuncPurity, FuncShape};
pub use r#if::If;
pub use item_shape::{ItemShape, ItemShapeMut};
pub use r#let::{Let, LetOrigin, TrivialLet};
pub use r#match::{Match, MatchArm};
pub use module::Module;
pub use path::Path;
pub use pattern::{Pattern, PatternKind, StructFieldPattern};
pub use poly::Poly;
pub use prelude::{PRELUDES, use_prelude};
pub use session::Session;
pub use r#struct::{Struct, StructField, StructInitField, StructShape};
pub use r#type::{Type, TypeAssertion};
pub use r#use::Use;

pub use sodigy_parse::Generic;

pub fn lower(parse_session: ParseSession) -> Session {
    let mut session = Session::from_parse_session(&parse_session);
    let mut top_level_block = match Block::from_ast(&parse_session.ast, &mut session) {
        Ok(block) => block,
        Err(()) => {
            return session;
        },
    };

    for r#let in top_level_block.lets.drain(..) {
        session.lets.push(r#let);
    }

    for assert in top_level_block.asserts.drain(..) {
        session.asserts.push(assert);
    }

    session.substitute_closures();
    session
}
