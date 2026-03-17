use crate::{RenderSpanSession, Span};

// It's used for more helpful error messages.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SpanDeriveKind {
    // It's a derived span, but is trivial that the error message doesn't have to mention that it's derived.
    Trivial,

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

    // When a function is monomorphized, *every* span in the function are derived.
    // Each monomorphization has a unique id, which helps identifying function and
    // generating error messages. Each session manages the ids.
    Monomorphize(u128),
}

impl SpanDeriveKind {
    // It returns None if the error note is too obvious.
    pub fn error_note(&self, session: &mut RenderSpanSession) -> Option<String> {
        match self {
            SpanDeriveKind::Trivial => None,
            SpanDeriveKind::Pipeline => Some(String::from("It is desugared to an inline `let` statement.")),
            SpanDeriveKind::ConstEval => Some(String::from("It is evaluated at compile-time.")),
            SpanDeriveKind::ExprInPattern => Some(String::from("It is desugared to a guard expression.")),
            SpanDeriveKind::Lambda => None,
            SpanDeriveKind::IfLet => Some(String::from("It is desugared to a match expression.")),
            SpanDeriveKind::FuncDefaultValue => Some(String::from("It is desugared to a `let` statement.")),
            SpanDeriveKind::MatchScrutinee(_) => None,
            SpanDeriveKind::ConcatPatternRest => Some(String::from("It is desugared to a rest pattern.")),
            SpanDeriveKind::ConcatPatternList => Some(String::from("It is desugared to a list pattern.")),
            SpanDeriveKind::FStringToString => Some(String::from("It is desugared to `to_string(..)`.")),
            SpanDeriveKind::FStringConcat => Some(String::from("It is desugared to a `++` operator.")),
            SpanDeriveKind::ConvertError => None,
            SpanDeriveKind::Monomorphize(id) => {
                // TODO: `unwrap()` vs returning None
                let mono_info = session.get_monomorphization_info(*id).unwrap();
                Some(format!("This is inside a monomorphization of `{}`.", mono_info.info))
            },
        }
    }
}

impl Span {
    #[must_use = "method returns a new span and does not mutate the original span"]
    pub fn derive(&self, kind: SpanDeriveKind) -> Span {
        match self {
            Span::None => Span::None,
            // TODO: If it derives a derived-span, the previous information is gone!
            //       But it would be toooo tricky to track the history of derivations...
            Span::Range { file, start, end } | Span::Derived { file, start, end, .. } => Span::Derived {
                kind,
                file: *file,
                start: *start,
                end: *end,
            },
            _ => panic!("TODO: {self:?}, {kind:?}"),
        }
    }

    #[must_use = "method returns a new span and does not mutate the original span"]
    pub fn monomorphize(&self, id: u128) -> Span {
        match self {
            Span::Poly { name, kind, .. } => Span::Poly {
                name: *name,
                kind: *kind,
                monomorphize_id: Some(id),
            },
            _ => self.derive(SpanDeriveKind::Monomorphize(id)),
        }
    }
}
