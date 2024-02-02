use crate::prelude::uids;
use crate::ty::Type;
use sodigy_intern::{
    InternedNumeric,
    InternedString,
    intern_numeric_u32,
    unintern_string,
};
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

mod endec;
pub mod lower;

#[derive(Clone, Hash)]
pub struct Expr {
    pub(crate) kind: ExprKind,
    pub(crate) ty: Type,
    pub(crate) span: SpanRange,
}

impl Expr {
    pub fn new_int(n: InternedNumeric) -> Self {
        Expr {
            kind: ExprKind::Integer(n),
            ty: Type::Solid(uids::INT_DEF),
            span: SpanRange::dummy(0x094b82b7),
        }
    }

    pub fn new_char(c: char) -> Self {
        Expr {
            kind: ExprKind::Integer(intern_numeric_u32(c as u32)),
            ty: Type::Solid(uids::CHAR_DEF),
            span: SpanRange::dummy(0xe1efa8eb),
        }
    }

    // String = List(Char)
    pub fn new_string(s: &InternedString) -> Self {
        // it's guaranteed to be a valid UTF-8
        let s = unsafe { String::from_utf8_unchecked(unintern_string(*s).to_vec()) };
        let args = s.chars().map(|c| Expr::new_char(c)).collect();

        Expr {
            kind: ExprKind::Call {
                f: uids::LIST_INIT,
                args,
            },
            ty: Type::Solid(uids::STRING_DEF),
            span: SpanRange::dummy(0xed8d9b6a),
        }
    }

    // Bytes = List(Byte)
    pub fn new_bytes(s: &InternedString) -> Self {
        let s = unintern_string(*s);
        let args = s.iter().map(|c| Expr::new_byte(*c)).collect();

        Expr {
            kind: ExprKind::Call {
                f: uids::LIST_INIT,
                args,
            },
            ty: Type::Solid(uids::BYTES_DEF),
            span: SpanRange::dummy(0x56d51634),
        }
    }

    pub fn new_byte(c: u8) -> Self {
        Expr {
            kind: ExprKind::Integer(intern_numeric_u32(c as u32)),
            ty: Type::Solid(uids::BYTE_DEF),
            span: SpanRange::dummy(0x964de250),
        }
    }

    pub fn set_span(&mut self, span: SpanRange) -> &mut Self {
        self.span = span;

        self
    }

    // it returns false if the type is not infered yet
    pub fn is_obviously_string(&self) -> bool {
        todo!()
    }
}

#[derive(Clone, Hash)]
pub enum ExprKind {
    Global(Uid),
    LocalNameBinding(LocalNameBinding),

    // TODO: other than `InternedNumeric`?
    Integer(InternedNumeric),
    Call {
        f: Uid,
        args: Vec<Expr>,
    },
    DynCall {
        f: Box<Expr>,
        args: Vec<Expr>,
    },
    Branch(Vec<BranchArm>),
}

#[derive(Clone, Hash)]
enum LocalNameBinding {
    // It assumes that every expr belongs to exactly one `Def`.
    // With that assumption, func args can be distinguished only
    // using an integer. If exprs from a specific `Def` are mixed
    // with other exprs from another `Def`, ... then what?
    // TODO: how should i implement function inlining?
    FuncArg(usize),
    FuncGeneric(usize),

    // TODO: scoped lets
}

#[derive(Clone, Hash)]
struct BranchArm {
    pub(crate) cond: Option<Expr>,
    pub(crate) value: Expr,
}
