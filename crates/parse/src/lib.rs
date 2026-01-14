use sodigy_lex::Session as LexSession;
use sodigy_span::Span;

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
mod lambda;
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
pub use attribute::{Attribute, Decorator, DocComment, DocCommentLine, Visibility};
pub use block::Block;
pub use r#enum::{Enum, EnumVariant, EnumVariantFields};
pub use expr::{Expr, ExprOrString, Field};
pub use func::{CallArg, Func, FuncParam};
pub use r#if::If;
pub use lambda::Lambda;
pub use r#let::Let;
pub use r#match::{Match, MatchArm};
pub use module::Module;
pub use pattern::{Pattern, PatternKind, PatternValueKind, RestPattern};
pub(crate) use pattern::ParsePatternContext;
pub use session::Session;
pub use r#struct::{Struct, StructField, StructInitField};
pub(crate) use tokens::Tokens;
pub use r#type::{Generic, Type};
pub use r#use::Use;

pub fn parse(lex_session: LexSession) -> Session {
    let mut session = Session::from_lex_session(&lex_session);
    let last_span = lex_session.tokens.last().map(|t| t.span.end()).unwrap_or(Span::None);
    let mut tokens = Tokens::new(&lex_session.tokens, last_span, &lex_session.intermediate_dir);
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

    if let Err(errors) = ast.check(true /* is_top_level */, &session.intermediate_dir) {
        session.errors = errors;
    }

    session.ast = ast;
    session
}
