use smallvec::{SmallVec, smallvec};
use sodigy_ast as ast;
use sodigy_error::{
    ExtraErrorInfo,
    RenderError,
    SodigyError,
    SodigyErrorKind,
    Stage,
    concat_commas,
};
use sodigy_intern::{InternedString, InternSession};
use sodigy_parse::IdentWithSpan;
use sodigy_span::SpanRange;

mod endec;

#[derive(Clone)]
pub struct HirError {
    kind: HirErrorKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrorInfo,
}

impl HirError {
    pub fn name_collision(id1: IdentWithSpan, id2: IdentWithSpan) -> Self {
        HirError {
            kind: HirErrorKind::NameCollision(id1.id()),
            spans: smallvec![*id1.span(), *id2.span()],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn undefined_name(name: IdentWithSpan, suggestions: Vec<InternedString>) -> Self {
        HirError {
            kind: HirErrorKind::UndefinedName {
                name: name.id(),
                suggestions,
            },
            spans: smallvec![*name.span()],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn no_dependent_types(id: IdentWithSpan) -> Self {
        HirError {
            kind: HirErrorKind::NoDependentTypes(id.id()),
            spans: smallvec![*id.span()],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn undefined_deco(deco: IdentWithSpan) -> Self {
        HirError {
            kind: HirErrorKind::UndefinedDeco(deco.id()),
            spans: smallvec![*deco.span()],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn refutable_pattern_in_let(pattern: &ast::Pattern) -> Self {
        HirError {
            kind: HirErrorKind::RefutablePatternInLet,
            spans: smallvec![pattern.span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn open_inclusive_range(span: SpanRange) -> Self {
        HirError {
            kind: HirErrorKind::OpenInclusiveRange,
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn unmatchable_pattern(span: SpanRange) -> Self {
        HirError {
            kind: HirErrorKind::UnmatchablePattern,
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn multiple_shorthands(spans: Vec<SpanRange>) -> Self {
        HirError {
            kind: HirErrorKind::MultipleShorthands,
            spans: spans.into(),
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn inclusive_string_pattern(span: SpanRange) -> Self {
        HirError {
            kind: HirErrorKind::InclusiveStringPattern,
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn name_binding_not_allowed_here(span: SpanRange) -> Self {
        HirError {
            kind: HirErrorKind::NameBindingNotAllowedHere,
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn ty_anno_not_allowed_here(span: SpanRange) -> Self {
        HirError {
            kind: HirErrorKind::TyAnnoNotAllowedHere,
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn name_not_bound_in_all_patterns(
        name: IdentWithSpan,
        span_of_pattern_that_dont_have_the_name: SpanRange,
    ) -> Self {
        HirError {
            kind: HirErrorKind::NameNotBoundInAllPatterns(name.id()),
            spans: smallvec![*name.span(), span_of_pattern_that_dont_have_the_name],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn type_error(
        span: Vec<SpanRange>,
        expected: String,
        got: String,
    ) -> Self {
        HirError {
            kind: HirErrorKind::TypeError { expected, got },
            spans: span.into(),
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn todo(msg: &str, span: SpanRange) -> Self {
        HirError {
            kind: HirErrorKind::TODO(msg.to_string()),
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }
}

impl SodigyError<HirErrorKind> for HirError {
    fn get_mut_error_info(&mut self) -> &mut ExtraErrorInfo {
        &mut self.extra
    }

    fn get_error_info(&self) -> &ExtraErrorInfo {
        &self.extra
    }

    fn get_first_span(&self) -> Option<SpanRange> {
        self.spans.get(0).copied()
    }

    fn get_spans(&self) -> &[SpanRange] {
        &self.spans
    }

    fn error_kind(&self) -> &HirErrorKind {
        &self.kind
    }

    fn index(&self) -> u32 {
        4
    }

    fn get_stage(&self) -> Stage {
        Stage::Hir
    }
}

#[derive(Clone)]
pub enum HirErrorKind {
    NameCollision(InternedString),
    NoDependentTypes(InternedString),
    UndefinedName {
        name: InternedString,
        suggestions: Vec<InternedString>,
    },
    UndefinedDeco(InternedString),
    RefutablePatternInLet,
    OpenInclusiveRange,
    UnmatchablePattern,
    MultipleShorthands,
    InclusiveStringPattern,
    NameBindingNotAllowedHere,
    TyAnnoNotAllowedHere,
    NameNotBoundInAllPatterns(InternedString),

    // It's supposed to be equal to `MirErrorKind::TypeError`
    // TODO: how do I guarantee that?
    TypeError {
        expected: String,
        got: String,
    },
    TODO(String),
}

impl SodigyErrorKind for HirErrorKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            HirErrorKind::NameCollision(name) => format!("the name `{}` is bound multiple times", name.render_error()),
            HirErrorKind::UndefinedName { name, .. } => format!("undefined name `{}`", name.render_error()),
            HirErrorKind::NoDependentTypes(_) => String::from("dependent types not allowed"),
            HirErrorKind::UndefinedDeco(name) => format!("unknown decorator `{}`", name.render_error()),
            HirErrorKind::RefutablePatternInLet => String::from("refutable pattern in `let` statement"),
            HirErrorKind::OpenInclusiveRange => String::from("inclusive range with an open end"),
            HirErrorKind::UnmatchablePattern => String::from("unmatchable pattern"),
            HirErrorKind::MultipleShorthands => String::from("multiple shorthands"),
            HirErrorKind::InclusiveStringPattern => String::from("inclusive range in a string pattern"),
            HirErrorKind::NameBindingNotAllowedHere => String::from("name binding not allowed here"),
            HirErrorKind::TyAnnoNotAllowedHere => String::from("type annotation not allowed here"),
            HirErrorKind::NameNotBoundInAllPatterns(name) => format!("name `{}` not bound in all patterns", name.render_error()),
            HirErrorKind::TypeError {
                expected,
                got,
            } => format!("expected type `{expected}`, got type `{got}`"),
            HirErrorKind::TODO(s) => format!("not implemented: {s}"),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            HirErrorKind::UndefinedName {
                suggestions,
                ..
            } => match suggestions.len() {
                0 => String::new(),
                1 => format!(
                    "A similar name exists in the current scope: `{}`",
                    suggestions[0].render_error(),
                ),
                _ => format!(
                    "Similar names exist in the current scope: {}",
                    concat_commas(
                        &suggestions.iter().map(
                            |s| s.render_error()
                        ).collect::<Vec<String>>(),
                        "and",
                        "`",
                        "`",
                    ),
                ),
            },
            HirErrorKind::RefutablePatternInLet => String::from("TODO: explain what refutable patterns are."),
            HirErrorKind::UnmatchablePattern => String::from("Nothing can match this pattern."),
            HirErrorKind::MultipleShorthands => String::from("There can be at most one shorthand pattern."),
            HirErrorKind::NameNotBoundInAllPatterns(name) => format!("This pattern is missing the name binding `{}`", name.render_error()),
            HirErrorKind::NameCollision(_)
            | HirErrorKind::NoDependentTypes(_)
            | HirErrorKind::UndefinedDeco(_)
            | HirErrorKind::OpenInclusiveRange
            | HirErrorKind::InclusiveStringPattern
            | HirErrorKind::NameBindingNotAllowedHere
            | HirErrorKind::TyAnnoNotAllowedHere
            | HirErrorKind::TypeError { .. }
            | HirErrorKind::TODO(_) => String::new(),
        }
    }

    fn index(&self) -> u32 {
        match self {
            HirErrorKind::NameCollision(..) => 0,
            HirErrorKind::NoDependentTypes(..) => 1,
            HirErrorKind::UndefinedName { .. } => 2,
            HirErrorKind::UndefinedDeco(..) => 3,
            HirErrorKind::RefutablePatternInLet => 4,
            HirErrorKind::OpenInclusiveRange => 5,
            HirErrorKind::UnmatchablePattern => 6,
            HirErrorKind::MultipleShorthands => 7,
            HirErrorKind::InclusiveStringPattern => 8,
            HirErrorKind::NameBindingNotAllowedHere => 9,
            HirErrorKind::TyAnnoNotAllowedHere => 10,
            HirErrorKind::NameNotBoundInAllPatterns(_) => 11,
            HirErrorKind::TypeError { .. } => 12,
            HirErrorKind::TODO(..) => 63,
        }
    }
}
