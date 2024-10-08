use crate as hir;
use sodigy_ast as ast;
use sodigy_attribute::Attribute;
use sodigy_parse::IdentWithSpan;
use sodigy_uid::Uid;

mod endec;
mod fmt;
mod lower;

pub use lower::lower_ast_func;

#[derive(Clone)]
pub struct Func {
    pub name: IdentWithSpan,

    // None if incallable
    pub args: Option<Vec<Arg>>,
    pub generics: Vec<ast::GenericDef>,
    pub return_value: hir::Expr,
    pub return_type: Option<hir::Type>,
    pub attributes: Vec<Attribute<hir::Expr>>,
    pub uid: Uid,

    pub kind: FuncKind,
}

#[derive(Clone)]
pub struct Arg {
    pub name: IdentWithSpan,
    pub ty: Option<hir::Type>,
    pub has_question_mark: bool,
    pub attributes: Vec<Attribute<hir::Expr>>,
}

#[derive(Clone)]
pub enum FuncKind {
    Normal,  // ones defined by the user
    Lambda,
    Enum { variants: Vec<Uid> },
    EnumVariant { parent: Uid },
    StructDef,
    StructConstr,
}
