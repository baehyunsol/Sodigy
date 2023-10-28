use sodigy_ast::IdentWithSpan;
use sodigy_err::{ExtraErrInfo, SodigyError, SodigyErrorKind};
use sodigy_intern::{InternedString, InternSession};
use sodigy_span::SpanRange;

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

pub enum HirErrorKind {
    NameCollision(InternedString),
}

impl SodigyErrorKind for HirErrorKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            HirErrorKind::NameCollision(name) => format!("the name `{name}` is bound multiple times"),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            _ => String::new(),
        }
    }
}
