use crate::Session;
use sodigy_error::{Error, ErrorKind};
use sodigy_parse as ast;
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::{
    InternedString,
    intern_string,
    unintern_string,
};
use std::collections::hash_map::{Entry, HashMap};

// `ast::Attribute` is first lowered to this type. It does some basic
// checks (redundant names, undefined names, arguments).
// Each item extracts extra information from this type.
pub struct Attribute {
    pub doc_comment: Option<DocComment>,
    pub decorators: HashMap<Vec<InternedString>, Decorator>,
    pub public: Public,
}

impl Attribute {
    pub fn from_ast(ast_attribute: &ast::Attribute, session: &mut Session, rule: AttributeRule) -> Result<Attribute, ()> {
        let doc_comment = match (rule.doc_comment, ast_attribute.doc_comment) {
            (Requirement::Must, None) => todo!(),
            (Requirement::Never, Some(_)) => todo!(),
            _ => ast_attribute.doc_comment.clone(),
        };

        todo!()
    }
}

pub struct AttributeRule {
    pub doc_comment: Requirement,
    pub publicity: Requirement,
    pub decorators: HashMap<Vec<InternedString>, DecoratorRule>,
}

pub enum Requirement {
    Must,
    Maybe,
    Never,
}

pub struct DecoratorRule {
    pub name: Vec<InternedString>,
    pub requirement: Requirement,

    // `ArgCount::Zero` and `Requirement::Never` are different.
    // `ArgCount::Zero` is `@note()`, while `Requirement::Never` is `@note`.
    pub arg_requirement: Requirement,
    pub arg_count: ArgCount,
    pub arg_type: ArgType,

    pub keyword_args: HashMap<InternedString, ArgType>,
}

pub enum ArgType {
    StringLiteral,
    Expr,
}

pub enum ArgCount {
    Zero,
    Eq(usize),
    Gt(usize),
    Lt(usize),
}
