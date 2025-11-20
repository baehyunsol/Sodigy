use sodigy_name_analysis::Namespace;
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
mod attribute;
mod block;
mod endec;
mod r#enum;
mod expr;
mod func;
mod r#if;
mod r#let;
mod r#match;
mod module;
mod name;
mod pattern;
mod poly;
mod prelude;
mod session;
mod r#struct;
mod r#type;
mod r#use;

pub use alias::Alias;
pub use assert::Assert;
pub use attribute::{
    ArgCount,
    ArgType,
    Attribute,
    AttributeKind,
    AttributeRule,
    AttributeRuleKey,
    DecoratorRule,
    Requirement,
    Visibility,
};
pub use block::Block;
pub use r#enum::{Enum, EnumVariant, EnumVariantFields};
pub use expr::Expr;
pub use func::{CallArg, Func, FuncParam, FuncOrigin};
pub use r#if::If;
pub use r#let::{Let, LetOrigin};
pub use r#match::{Match, MatchBranch};
pub use module::Module;
pub use pattern::{FullPattern, Pattern};
pub use poly::Poly;
pub use session::Session;
pub use r#struct::{Struct, StructField, StructInitField};
pub use r#type::Type;
pub use r#use::Use;

pub use sodigy_parse::Generic;

pub fn lower(parse_session: ParseSession) -> Session {
    let mut session = Session::from_parse_session(&parse_session);
    let mut top_level_block = match Block::from_ast(
        &parse_session.ast,
        &mut session,
        true, // is_top_level
    ) {
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

    // std does not have preludes
    if !session.is_std {
        let Namespace::Block { names } = session.name_stack.pop().unwrap() else { unreachable!() };

        // If the code ever mentions `Int`, we have to add `use std.preludes.Int;`.
        for (name, (_, _, count)) in names.iter() {
            if !count.never_ever() {
                session.uses.push(prelude::use_prelude(*name));
            }
        }
    }

    session
}
