use crate::{
    ExprKind,
    HirError,
    HirSession,
    HirWarning,
    Type,
};
use sodigy_ast as ast;
use sodigy_intern::{InternedNumeric, InternedString, InternSession};
use sodigy_parse::IdentWithSpan;
use sodigy_session::SodigySession;
use sodigy_span::SpanRange;

mod endec;
mod fmt;
mod lower;
pub mod string;

pub use lower::{
    check_names_in_or_patterns,
    lower_ast_pattern,
    lower_patterns_to_name_bindings,
};
pub use string::StringPattern;

#[derive(Clone)]
pub struct Pattern {
    kind: PatternKind,
    span: SpanRange,
    ty: Option<Type>,
    bind: Option<IdentWithSpan>,
}

#[derive(Clone)]
pub enum PatternKind {
    Constant(ExprKind),
    Binding(InternedString),

    String(StringPattern),

    Range {
        ty: RangeType,

        // both inclusive
        from: NumberLike,
        to: NumberLike,
    },

    Tuple(Vec<Pattern>),
    List(Vec<Pattern>),

    // it's for matching enum variants, including ones that do not have any value
    // e.g. `Option.Some(x)`, `Option.None` and `None`
    TupleStruct {
        name: ast::DottedNames,
        fields: Vec<Pattern>,
    },

    Wildcard,   // _
    Shorthand,  // ..

    // invariant: len > 1
    Or(Vec<Pattern>),
}

// `let pattern PAT = EXPR;` is destructured to multiple `DestructuredPattern`s.
#[derive(Debug)]
pub struct DestructuredPattern {
    pub(crate) name: IdentWithSpan,

    // these are lowered later
    pub(crate) expr: ast::Expr,
    pub(crate) ty: Option<ast::TypeDef>,

    // if this name binding is defined by the programmer, it's true
    pub(crate) is_real: bool,
}

impl DestructuredPattern {
    pub fn new(name: IdentWithSpan, expr: ast::Expr, ty: Option<ast::TypeDef>, is_real: bool) -> Self {
        DestructuredPattern { name, expr, ty, is_real }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RangeType {
    Integer,
    Char,
    Ratio,
}

impl RangeType {
    pub fn try_from_pattern(
        p: &ast::Pattern,
        session: &mut HirSession,
    ) -> Result<Self, ()> {
        match &p.kind {
            ast::PatternKind::Number(num) => if num.is_integer() {
                Ok(RangeType::Integer)
            } else {
                Ok(RangeType::Ratio)
            },
            ast::PatternKind::Char(_) => Ok(RangeType::Char),
            _ => {
                session.push_error(HirError::type_error(
                    vec![p.span],

                    // TODO: better representation?
                    //       there's no standard for this kinda type annotations
                    String::from("Int | Ratio | Char"),  // expected
                    p.get_type_string(),  // got
                ));
                return Err(());
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumberLike {
    OpenEnd {
        is_negative: bool,
    },
    Exact(InternedNumeric),

    // case of `x` in `..x` when `x` is not an integer
    MinusEpsilon(InternedNumeric),
}

impl NumberLike {
    pub fn try_from_pattern(
        p: &ast::Pattern,
        session: &mut HirSession,
        inclusive: bool,
    ) -> Result<Self, ()> {
        match &p.kind {
            ast::PatternKind::Number(num) => if inclusive {
                Ok(NumberLike::Exact(*num))
            } else {
                if num.is_integer() {
                    let num = session.unintern_numeric(*num);
                    let new_num = num.minus_one();

                    Ok(NumberLike::Exact(session.intern_numeric(new_num)))
                } else {
                    Ok(NumberLike::MinusEpsilon(*num))
                }
            },
            ast::PatternKind::Char(c) => {
                let mut c = *c as u32;

                if inclusive {
                    if c == 0 {
                        session.push_error(HirError::unmatchable_pattern(p.span));
                        return Err(());
                    }

                    c -= 1;
                }

                Ok(NumberLike::Exact(session.intern_numeric(c.into())))
            },
            _ => {
                session.push_error(HirError::type_error(
                    vec![p.span],

                    // TODO: better representation?
                    //       there's no standard for this kinda type annotations
                    String::from("Int | Ratio | Char"),  // expected
                    p.get_type_string(),  // got
                ));
                return Err(());
            },
        }
    }

    pub fn zero() -> Self {
        NumberLike::Exact(InternedNumeric::zero())
    }

    pub fn is_minus_epsilon(&self) -> bool {
        matches!(self, NumberLike::MinusEpsilon(..))
    }

    pub fn try_into_u32(&self, session: &mut InternSession) -> Option<u32> {
        match self {
            NumberLike::Exact(num) => match u32::try_from(session.unintern_numeric(*num)) {
                Ok(n) => Some(n),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn gt(&self, other: &Self) -> bool {
        match (self, other) {
            (
                NumberLike::OpenEnd { is_negative: neg1 },
                NumberLike::OpenEnd { is_negative: neg2 },
            ) => match (*neg1, *neg2) {
                // we cannot compare infs
                (true, true) | (false, false) => false,
                (_, n2) => n2,
            },
            (
                NumberLike::OpenEnd { is_negative },
                _,
            ) => !*is_negative,
            (
                _,
                NumberLike::OpenEnd { is_negative },
            ) => *is_negative,
            (
                NumberLike::Exact(num1),
                NumberLike::Exact(num2),
            ) | (
                NumberLike::MinusEpsilon(num1),
                NumberLike::MinusEpsilon(num2),
            ) => num1.gt(num2),
            (
                NumberLike::Exact(num1),
                NumberLike::MinusEpsilon(num2),
            ) => if num1 == num2 {
                true
            } else {
                num1.gt(num2)
            },
            (
                NumberLike::MinusEpsilon(num1),
                NumberLike::Exact(num2),
            ) => if num1 == num2 {
                false
            } else {
                num1.gt(num2)
            },
        }
    }
}

pub fn check_range_pattern(
    p: &PatternKind,
    span: SpanRange,
    session: &mut HirSession,
) -> Result<(), ()> {
    match p {
        PatternKind::Range {
            from, to, ty,
        } => {
            if from == to {
                session.push_warning(
                    HirWarning::point_range(*from, *to, *ty, span)
                );
            }

            else if from.gt(to) {
                session.push_error(
                    HirError::unmatchable_pattern(span)
                );

                return Err(());
            }
        },
        _ => unreachable!(),
    }

    Ok(())
}
