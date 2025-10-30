use sodigy_lex::Session as LexSession;
use sodigy_span::Span;
use sodigy_string::intern_string;

mod alias;
mod assert;
mod attribute;
mod block;
mod check;
mod endec;
mod r#enum;
mod expr;
mod func;
mod r#if;
mod r#let;
mod r#match;
mod module;
mod pattern;
mod session;
mod r#struct;
mod tokens;
mod r#type;
mod r#use;

pub use alias::Alias;
pub use assert::Assert;
pub use attribute::{Attribute, Decorator, DocComment};
pub use block::Block;
pub use r#enum::Enum;
pub use expr::{Expr, Field};
pub use func::{CallArg, Func, FuncArgDef};
pub use r#if::If;
pub use r#let::Let;
pub use r#match::{Match, MatchBranch};
pub use module::Module;
pub use pattern::{FullPattern, Pattern};
pub use session::Session;
pub use r#struct::{Struct, StructInitField};
pub(crate) use tokens::Tokens;
pub use r#type::{GenericDef, Type};
pub use r#use::Use;

pub fn parse(lex_session: LexSession) -> Session {
    let mut session = Session::from_lex_session(&lex_session);
    let last_span = lex_session.tokens.last().map(|t| t.span.end()).unwrap_or(Span::None);
    let mut tokens = Tokens::new(&lex_session.tokens, last_span);
    let ast = match tokens.parse_block(
        true, // top-level
        Span::file(session.file),
    ) {
        Ok(ast) => ast,
        Err(errors) => {
            session.errors = errors;
            return session;
        },
    };

    if let Err(errors) = ast.check(true /* top_level */, &session) {
        session.errors = errors;
    }

    let main = intern_string(b"main", &session.intermediate_dir).unwrap();

    for func in ast.funcs.iter() {
        if func.name == main {
            session.main_func = Some(func.name_span);
            break;
        }
    }

    session.ast = ast;
    session
}
