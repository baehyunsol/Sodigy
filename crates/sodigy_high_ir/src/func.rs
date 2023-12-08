use crate as hir;
use sodigy_ast::{self as ast, IdentWithSpan};
use sodigy_intern::InternedString;
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
    pub(crate) generics: Vec<ast::GenericDef>,
    pub return_val: hir::Expr,
    pub return_ty: Option<hir::Type>,
    pub decorators: FuncDeco,
    pub(crate) doc: Option<InternedString>,
    pub uid: Uid,

    pub kind: FuncKind,
}

#[derive(Clone)]
pub struct Arg {
    pub name: IdentWithSpan,
    pub ty: Option<hir::Type>,
    pub has_question_mark: bool,
}

#[derive(Clone)]
pub enum FuncKind {
    Normal,  // ones defined by the user
    Lambda,
    Enum { variants: Vec<Uid> },
    EnumVariant { parent: Uid },
    StructConstr,
}

// lowered ast::Deco
// some simple decorators are interpreted and consumed!
#[derive(Clone, Default)]
pub struct FuncDeco {
    publicity: Publicity,

    // exprs in `@test.eq()`
    test_eq: Vec<hir::Expr>,
}

impl FuncDeco {
    // decorators for lambda functions
    pub fn default_lambda() -> Self {
        FuncDeco {
            publicity: Publicity::Private,
            test_eq: vec![],
        }
    }
}

#[derive(Clone, Default)]
enum Publicity {
    #[default]
    Public,
    Private
}
