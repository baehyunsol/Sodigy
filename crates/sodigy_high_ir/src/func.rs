use crate as hir;
use sodigy_ast::{self as ast, IdentWithSpan};
use sodigy_intern::InternedString;
use sodigy_uid::Uid;

mod lower;

pub use lower::lower_ast_func;

pub struct Func {
    name: IdentWithSpan,
    args: Option<Vec<Arg>>,
    generics: Vec<ast::GenericDef>,
    ret_val: hir::Expr,
    ret_ty: Option<hir::Type>,
    decorators: FuncDeco,
    doc: InternedString,
    uid: Uid,
}

pub struct Arg {
    name: IdentWithSpan,
    ty: Option<hir::Type>,
    has_question_mark: bool,
}

// lowered ast::Deco
// some simple decorators are interpreted and consumed!
pub struct FuncDeco {}
