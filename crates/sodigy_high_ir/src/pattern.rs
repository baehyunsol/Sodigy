use crate::{HirError, HirSession, HirWarning, Type};
use sodigy_ast::{self as ast, IdentWithSpan};
use sodigy_intern::{InternedNumeric, InternedString, InternSession};
use sodigy_number::SodigyNumber;
use sodigy_span::SpanRange;

mod endec;
mod fmt;
mod lower;

pub use lower::{lower_ast_pattern, lower_patterns_to_name_bindings};

#[derive(Clone)]
pub struct Pattern {
    kind: PatternKind,
    span: SpanRange,
    ty: Option<Type>,
    bind: Option<IdentWithSpan>,
}

#[derive(Clone)]
pub enum PatternKind {
    Binding(InternedString),

    Range {
        ty: RangeType,

        // both inclusive
        from: NumberLike,
        to: NumberLike,
    },
}

// `let pattern PAT = EXPR;` is destructured to multiple `DestructuredPattern`s.
pub struct DestructuredPattern {
    name: IdentWithSpan,

    // these are lowered later
    expr: ast::Expr,
    ty: Option<ast::TypeDef>,

    // if this name binding is defined by the programmer, it's true
    is_real: bool,
}

impl DestructuredPattern {
    pub fn new(name: IdentWithSpan, expr: ast::Expr, ty: Option<ast::TypeDef>, is_real: bool) -> Self {
        DestructuredPattern { name, expr, ty, is_real }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RangeType {
    Integer, Char,
    Ratio,
}

impl RangeType {
    pub fn try_from_pattern(
        p: &ast::Pattern,
        session: &mut HirSession,
    ) -> Result<Self, ()> {
        match &p.kind {
            ast::PatternKind::Number { num, .. } => if num.is_integer() {
                Ok(RangeType::Integer)
            } else {
                Ok(RangeType::Ratio)
            },
            ast::PatternKind::Char(_) => Ok(RangeType::Char),
            _ => {
                session.push_error(HirError::ty_error(vec![p.span]));
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
    Exact {
        num: InternedNumeric,
        is_negative: bool,
    },

    // case of `x` in `..x` when `x` is not an integer
    MinusEpsilon {
        num: InternedNumeric,
        is_negative: bool,
    },
}

impl NumberLike {
    pub fn try_from_pattern(
        p: &ast::Pattern,
        session: &mut HirSession,
        inclusive: bool,
    ) -> Result<Self, ()> {
        match &p.kind {
            ast::PatternKind::Number { num, is_negative } => if inclusive {
                Ok(NumberLike::Exact { num: *num, is_negative: *is_negative })
            } else {
                if num.is_integer() {
                    let num = session.unintern_numeric(*num);
                    let (new_num, is_negative) = SodigyNumber::minus_one(num.unwrap().clone(), *is_negative);

                    Ok(NumberLike::Exact {
                        num: session.intern_numeric(new_num),
                        is_negative,
                    })
                } else {
                    Ok(NumberLike::MinusEpsilon {
                        num: *num,
                        is_negative: *is_negative,
                    })
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

                Ok(NumberLike::Exact {
                    num: session.intern_numeric(c.into()),
                    is_negative: false,
                })
            },
            _ => {
                session.push_error(HirError::ty_error(vec![p.span]));
                return Err(());
            },
        }
    }

    pub fn zero() -> Self {
        NumberLike::Exact {
            num: InternedNumeric::zero(),
            is_negative: false,
        }
    }

    pub fn try_into_u32(&self, session: &mut InternSession) -> Option<u32> {
        match self {
            NumberLike::Exact {
                num, is_negative: false,
            } => match session.unintern_numeric(*num) {
                Some(n) => match u32::try_from(n) {
                    Ok(n) => Some(n),
                    _ => None,
                },
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
                _
            ) => !*is_negative,
            (
                _,
                NumberLike::OpenEnd { is_negative },
            ) => *is_negative,
            (
                NumberLike::Exact { num: num1, is_negative: neg1 },
                NumberLike::Exact { num: num2, is_negative: neg2 },
            ) | (
                NumberLike::MinusEpsilon { num: num1, is_negative: neg1 },
                NumberLike::MinusEpsilon { num: num2, is_negative: neg2 },
            ) | (
                NumberLike::Exact { num: num1, is_negative: neg1 },
                NumberLike::MinusEpsilon { num: num2, is_negative: neg2 },
            ) | (
                NumberLike::MinusEpsilon { num: num1, is_negative: neg1 },
                NumberLike::Exact { num: num2, is_negative: neg2 },
            ) => if *neg1 != *neg2 {
                *neg2
            }

            // we have to do our best to avoid calling `num1.gt(num2)`
            else if *num1 == *num2 {
                let is_exact1 = matches!(self, NumberLike::Exact { .. });
                let is_exact2 = matches!(other, NumberLike::Exact { .. });

                if is_exact1 == is_exact2 {
                    true
                }

                // is_exact1      is_exact2      neg1      neg2      gt
                //    true          false        true      true     false
                //    true          false        false     false    true
                //    false         true         true      true     true
                //    false         true         false     false    false
                // gt = is_exact1 ^ neg1
                else {
                    is_exact1 ^ *neg1
                }
            }

            else {
                num1.gt(num2) ^ *neg1
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
