use super::SodigyError;
use crate::expr::Expr;
use crate::session::LocalParseSession;
use crate::span::Span;

pub struct TypeError {
    kind: TypeErrorKind,
    span: Span,
    expected: TypeNameOrExpr,
    got: TypeNameOrExpr,
}

impl TypeError {
    pub fn branch_no_boolean(span: Span, got: String) -> Self {
        TypeError {
            kind: TypeErrorKind::BranchConditionNotBoolean,
            span,
            expected: TypeNameOrExpr::None,  // Trivial
            got: TypeNameOrExpr::TypeName(got),
        }
    }

    pub fn type_anno_not_type(span: Span, got: String) -> Self {
        TypeError {
            kind: TypeErrorKind::TypeAnnoNotType,
            span,
            expected: TypeNameOrExpr::None,  // Trivial
            got: TypeNameOrExpr::TypeName(got),
        }
    }

    pub fn wrong_number_of_arg(span: Span, expected_num: usize, got_num: usize) -> Self {
        TypeError {
            kind: TypeErrorKind::WrongNumberOfArg(expected_num, got_num),
            span,
            expected: TypeNameOrExpr::None,
            got: TypeNameOrExpr::None,
        }
    }

    pub fn not_callable(span: Span, got: String) -> Self {
        TypeError {
            kind: TypeErrorKind::NotCallable,
            span,
            expected: TypeNameOrExpr::None,  // Trivial
            got: TypeNameOrExpr::TypeName(got),
        }
    }

    pub fn wrong_func_arg(span: Span, expected: String, got: String) -> Self {
        TypeError {
            kind: TypeErrorKind::WrongFuncArg,
            span,
            expected: TypeNameOrExpr::TypeName(expected),
            got: TypeNameOrExpr::TypeName(got),
        }
    }
}

impl SodigyError for TypeError {
    // doesn't impl `render_err` for `TypeErrorKind`, because many
    // `expected`s are dependent on `TypeErrorKind`
    fn render_err(&self, session: &LocalParseSession) -> String {
        match self.kind {
            TypeErrorKind::BranchConditionNotBoolean => format!(
                "Error: non-boolean condition in a branch expression\nExpected `Boolean`, got `{}`.\n{}",
                self.got.unwrap_type_name(),
                self.span.render_err(session),
            ),
            _ => todo!(),
        }
    }

    fn try_add_more_helpful_message(&mut self) {
        // Nothing to do
    }

    fn get_first_span(&self) -> Span {
        self.span
    }
}

enum TypeErrorKind {
    /// when a condition of a branch is not Boolean
    BranchConditionNotBoolean,

    /// `if cond { 3 } else { '4' }`
    BranchDifferentTypes,

    /// when `foo` in `foo()` is not callable
    NotCallable,

    /// in `foo(x: A, y: B): C`, `A`, `B`, and `C` must be types.
    TypeAnnoNotType,

    /// `usize` is the index of the missing arg\
    /// for now, it only works when there's 1 missing arg
    MissingFuncArg(usize),

    /// `usize` is the index of the unexpected arg\
    /// for now, it only works when there's 1 unexpected arg
    UnexpectedFuncArg(usize),

    /// it represents the case where the number of
    /// the args is correct, but the type(s) are incorrect\
    /// if there are multiple incorrect types, emit this error
    /// multiple times
    WrongFuncArg,

    /// if the difference is bigger than 1, this kind is used\
    /// the first `usize` is the expected number, while the second
    /// one is the given one
    WrongNumberOfArg(usize, usize),
}

enum TypeNameOrExpr {
    TypeName(String),
    Expr(Expr),
    None,
}

impl TypeNameOrExpr {
    pub fn unwrap_type_name(&self) -> String {
        match self {
            TypeNameOrExpr::TypeName(s) => s.to_string(),
            _ => panic!("Internal Compiler Error 0B9CDCBB5B8"),
        }
    }
}
