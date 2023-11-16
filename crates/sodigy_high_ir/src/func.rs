use crate as hir;
use sodigy_ast::{self as ast, IdentWithSpan};
use sodigy_intern::InternedString;
use sodigy_uid::Uid;

mod fmt;
mod lower;

pub use lower::lower_ast_func;

pub struct Func {
    pub name: IdentWithSpan,
    pub args: Option<Vec<Arg>>,
    pub(crate) generics: Vec<ast::GenericDef>,
    pub ret_val: hir::Expr,
    pub ret_ty: Option<hir::Type>,
    pub decorators: FuncDeco,
    pub(crate) doc: Option<InternedString>,
    pub(crate) uid: Uid,
}

#[derive(Clone)]
pub struct Arg {
    pub name: IdentWithSpan,
    pub ty: Option<hir::Type>,
    pub has_question_mark: bool,
}

// lowered ast::Deco
// some simple decorators are interpreted and consumed!
#[derive(Default)]
pub struct FuncDeco {
    publicity: Publicity,
}

impl FuncDeco {
    // decorators for lambda functions
    pub fn default_lambda() -> Self {
        FuncDeco {
            publicity: Publicity::Private,
        }
    }
}

#[derive(Default)]
enum Publicity {
    #[default]
    Public,
    Private
}
