use sodigy_ast::IdentWithSpan;
use sodigy_err::{concat_commas, ExtraErrInfo, SodigyError, SodigyErrorKind};
use sodigy_intern::{InternedString, InternSession};
use sodigy_span::SpanRange;

#[derive(Clone)]
pub struct HirError {
    kind: HirErrorKind,
    spans: Vec<SpanRange>,
    extra: ExtraErrInfo,
}

impl HirError {
    pub fn name_collision(id1: IdentWithSpan, id2: IdentWithSpan) -> Self {
        HirError {
            kind: HirErrorKind::NameCollision(*id1.id()),
            spans: vec![*id1.span(), *id2.span()],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn undefined_name(name: IdentWithSpan, suggestions: Vec<InternedString>) -> Self {
        HirError {
            kind: HirErrorKind::UndefinedName {
                name: *name.id(),
                suggestions,
            },
            spans: vec![*name.span()],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn no_dependent_types(id: IdentWithSpan) -> Self {
        HirError {
            kind: HirErrorKind::NoDependentTypes(*id.id()),
            spans: vec![*id.span()],
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
}

impl SodigyErrorKind for HirErrorKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            HirErrorKind::NameCollision(name) => format!("the name `{name}` is bound multiple times"),
            HirErrorKind::UndefinedName { name, .. } => format!("undefined name `{name}`"),
            HirErrorKind::NoDependentTypes(_) => format!("dependent types not allowed"),
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
            _ => String::new(),
        }
    }
}
