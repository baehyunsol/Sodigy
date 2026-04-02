use crate::{RenderSpanSession, Span};

// It's used for more helpful error messages.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SpanDeriveKind {
    // `a |> f($, b)` -> `{ let $0 = a; f(_, b) }`
    Pipeline,

    // A result of a compile-time-constant-evaluation.
    ConstEval,

    // `match foo { x => 0 }` -> `match foo { $tmp if tmp == x => 0 }`
    // `match foo { x + 1 => 0 }` -> `match foo { $tmp if tmp == x + 1 => 0 }`
    ExprInPattern,

    // `let f = \() => 0;` -> `fn lambda() = 0; let f = lambda;`
    Lambda,

    // `if let Some(x) = foo() { .. }` -> `match foo() { Some(x) => { .. }, .. }`
    IfLet,

    // `let ($x, _, $y) = foo();` -> `let tmp = match foo() { ($x, _, $y) => (x, y) }; let x = tmp._0; let y = tmp._1;`
    LetPattern(u32),

    // `fn add(a, b=1) = a + b;` -> `let b_default = 1; fn add(a, b=b_default) = a + b;`
    FuncDefaultValue,

    // `match (x, y) { .. }` -> `let scrutinee = (x, y); match scrutinee { .. }`
    MatchScrutinee(u32),

    // `[n] ++ ns` -> `[n, ns @ ..]`
    ConcatPatternRest,
    ConcatPatternList,

    // `f"{x} + {y}"` -> `to_string(x) ++ " + " ++ to_string(y)`
    FStringToString,
    FStringConcat,

    // `"3" as? <Int>` -> `std.convert.try_convert.<_, Int, _>("3")`
    // The first `_` is not derived, and the second `_` is derived with this.
    ConvertError,
}

impl SpanDeriveKind {
    // It returns None if the error note is too obvious.
    pub fn error_note(&self, session: &mut RenderSpanSession) -> Option<&'static str> {
        match self {
            SpanDeriveKind::Pipeline => Some("It is desugared to an inline `let` statement."),
            SpanDeriveKind::ConstEval => Some("It is evaluated at compile-time."),
            SpanDeriveKind::ExprInPattern => Some("It is desugared to a guard expression."),
            SpanDeriveKind::Lambda => None,
            SpanDeriveKind::IfLet => Some("It is desugared to a match expression."),

            // We have a lot of error variants for let-patterns, so we don't need an extra note.
            SpanDeriveKind::LetPattern(_) => None,

            SpanDeriveKind::FuncDefaultValue => Some("It is desugared to a `let` statement."),
            SpanDeriveKind::MatchScrutinee(_) => None,
            SpanDeriveKind::ConcatPatternRest => Some("It is desugared to a rest pattern."),
            SpanDeriveKind::ConcatPatternList => Some("It is desugared to a list pattern."),
            SpanDeriveKind::FStringToString => Some("It is desugared to `convert.<_, String>(..)`."),
            SpanDeriveKind::FStringConcat => Some("It is desugared to a `++` operator."),
            SpanDeriveKind::ConvertError => None,
        }
    }
}

impl Span {
    #[must_use = "method returns a new span and does not mutate the original span"]
    pub fn derive(&self, kind: SpanDeriveKind) -> Span {
        match self {
            Span::None => Span::None,
            span => Span::Derived {
                kind,
                span: Box::new(span.clone()),
            },
        }
    }

    #[must_use = "method returns a new span and does not mutate the original span"]
    pub fn monomorphize(&self, id: u64) -> Span {
        match self {
            Span::None => Span::None,
            span => Span::Monomorphize {
                id,
                span: Box::new(span.clone()),
            },
        }
    }
}
