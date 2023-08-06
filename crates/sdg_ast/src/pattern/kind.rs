use super::{FieldPatternDef, Pattern};
use crate::expr::Expr;
use crate::session::InternedString;
use crate::token::Token;

#[derive(Clone)]
pub enum PatternKind {
    WildCard,         // _
    Shorthand,        // ..  // only valid inside slices, tuples and structs
    Constant(Token),  // only int, real, and string

    // 1..10     -> only integers, no reals
    // 'a'..'z'
    // 1..~10    -> inclusive
    // 1..
    // ..100     -> can open either end (not both)
    Range(Option<Token>, Option<Token>, RangeType),

    // it's a subset of `Expr`
    // Value::Identifier, Value::Object, or Path
    Identifier(Box<Expr>),  // a.b.c
    Binding(InternedString),    // $a
    Tuple(Vec<Pattern>),    // ($a, $b, .., $c)
    Slice(Vec<Pattern>),    // [$a, $b, .., $c]

    // `Box<Expr>` of enums and structs is a subset of `Expr`
    // Value::Identifier, Value::Object, or Path
    EnumTuple(Box<Expr>, Vec<Pattern>),  // a.b.c($a, $b, $c)

    // (name_of_struct, field_defs, has_shorthand)
    Struct(Box<Expr>, Vec<FieldPatternDef>, bool),  // a.b.c { a: ($a, $b, $c), b: $b, c: _ }
}

#[derive(Copy, Clone, PartialEq)]
pub enum RangeType {
    Exclusive,
    Inclusive,
}

use std::fmt::{Display, Formatter, self};

impl Display for RangeType {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", match self {
            RangeType::Inclusive => "..~",
            RangeType::Exclusive => "..",
        })
    }
}
