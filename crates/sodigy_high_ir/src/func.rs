use crate as hir;
use sodigy_ast::{self as ast, IdentWithSpan};
use sodigy_intern::InternedString;
use sodigy_uid::Uid;

mod fmt;
mod lower;

pub use lower::lower_ast_func;

pub struct Func {
    name: IdentWithSpan,
    args: Option<Vec<Arg>>,
    generics: Vec<ast::GenericDef>,
    ret_val: hir::Expr,
    ret_ty: Option<hir::Type>,
    decorators: FuncDeco,
    doc: Option<InternedString>,
    uid: Uid,
}

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

#[derive(Default)]
enum Publicity {
    #[default]
    Public,
    Private
}
