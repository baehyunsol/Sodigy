use smallvec::{smallvec, SmallVec};
use sodigy_ast::{self as ast, IdentWithSpan};
use sodigy_err::{concat_commas, ExtraErrInfo, SodigyError, SodigyErrorKind};
use sodigy_intern::{InternedString, InternSession};
use sodigy_span::SpanRange;

#[derive(Clone)]
pub struct HirError {
    kind: HirErrorKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrInfo,
}

impl HirError {
    pub fn name_collision(id1: IdentWithSpan, id2: IdentWithSpan) -> Self {
        HirError {
            kind: HirErrorKind::NameCollision(*id1.id()),
            spans: smallvec![*id1.span(), *id2.span()],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn undefined_name(name: IdentWithSpan, suggestions: Vec<InternedString>) -> Self {
        HirError {
            kind: HirErrorKind::UndefinedName {
                name: *name.id(),
                suggestions,
            },
            spans: smallvec![*name.span()],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn no_dependent_types(id: IdentWithSpan) -> Self {
        HirError {
            kind: HirErrorKind::NoDependentTypes(*id.id()),
            spans: smallvec![*id.span()],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn undefined_deco(deco: IdentWithSpan) -> Self {
        HirError {
            kind: HirErrorKind::UndefinedDeco(*deco.id()),
            spans: smallvec![*deco.span()],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn refutable_pattern_in_let(pattern: &ast::Pattern) -> Self {
        HirError {
            kind: HirErrorKind::RefutablePatternInLet,
            spans: smallvec![pattern.span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn open_inclusive_range(span: SpanRange) -> Self {
        HirError {
            kind: HirErrorKind::OpenInclusiveRange,
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn unmatchable_pattern(span: SpanRange) -> Self {
        HirError {
            kind: HirErrorKind::UnmatchablePattern,
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    // tmp variant for type errors.
    // must be replaced with 'real' type errors when
    // Sodigy type system is implemented
    pub fn ty_error(span: Vec<SpanRange>) -> Self {
        HirError {
            kind: HirErrorKind::TyError,
            spans: span.into(),
            extra: ExtraErrInfo::none(),
        }   
    }

    pub fn todo(msg: &str, span: SpanRange) -> Self {
        HirError {
            kind: HirErrorKind::TODO(msg.to_string()),
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }
}

impl SodigyError<HirErrorKind> for HirError {
    fn get_mut_error_info(&mut self) -> &mut ExtraErrInfo {
        &mut self.extra
    }

    fn get_error_info(&self) -> &ExtraErrInfo {
        &self.extra
    }

    fn get_first_span(&self) -> SpanRange {
        self.spans[0]
    }

    fn get_spans(&self) -> &[SpanRange] {
        &self.spans
    }

    fn err_kind(&self) -> &HirErrorKind {
        &self.kind
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

    // tmp variant for type errors.
    // must be replaced with 'real' type errors when
    // Sodigy type system is implemented
    TyError,
    TODO(String),
}

impl SodigyErrorKind for HirErrorKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            HirErrorKind::NameCollision(name) => format!("the name `{name}` is bound multiple times"),
            HirErrorKind::UndefinedName { name, .. } => format!("undefined name `{name}`"),
            HirErrorKind::NoDependentTypes(_) => String::from("dependent types not allowed"),
            HirErrorKind::UndefinedDeco(name) => format!("unknown decorator `{name}`"),
            HirErrorKind::RefutablePatternInLet => String::from("refutable pattern in a `let` statement"),
            HirErrorKind::OpenInclusiveRange => String::from("inclusive range with an open end"),
            HirErrorKind::UnmatchablePattern => String::from("unmatchable pattern"),
            HirErrorKind::TyError => String::from("TODO: Type Error"),  // Sodigy type system is not complete yet
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
                    suggestions[0],
                ),
                _ => format!(
                    "Similar names exist in the current scope: {}",
                    concat_commas(
                        &suggestions.iter().map(
                            |s| format!("{s}")
                        ).collect::<Vec<String>>(),
                        "and",
                        "`",
                        "`",
                    ),
                ),
            },
            HirErrorKind::RefutablePatternInLet => String::from("TODO: explain what refutable patterns are."),
            HirErrorKind::UnmatchablePattern => String::from("Nothing can match this pattern."),
            _ => String::new(),
        }
    }
}
