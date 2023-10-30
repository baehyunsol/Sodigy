use crate::names::NameBindingType;
use sodigy_ast::IdentWithSpan;
use sodigy_err::{ExtraErrInfo, SodigyError, SodigyErrorKind};
use sodigy_intern::{InternedString, InternSession};
use sodigy_span::SpanRange;

pub struct HirWarning {
    kind: HirWarningKind,
    spans: Vec<SpanRange>,
    extra: ExtraErrInfo,
}

impl HirWarning {
    pub fn redef_prelude(id: IdentWithSpan) -> Self {
        HirWarning {
            kind: HirWarningKind::RedefPrelude(*id.id()),
            spans: vec![*id.span()],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn unused_name(id: IdentWithSpan, binding_type: NameBindingType) -> Self {
        HirWarning {
            kind: HirWarningKind::UnusedName(*id.id(), binding_type),
            spans: vec![*id.span()],
            extra: ExtraErrInfo::none(),
        }
    }
}

impl SodigyError<HirWarningKind> for HirWarning {
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

    fn err_kind(&self) -> &HirWarningKind {
        &self.kind
    }

    fn is_warning(&self) -> bool {
        true
    }
}

pub enum HirWarningKind {
    RedefPrelude(InternedString),
    UnusedName(InternedString, NameBindingType),
}

impl SodigyErrorKind for HirWarningKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            HirWarningKind::RedefPrelude(name) => format!("redefinition of prelude `{name}`"),
            HirWarningKind::UnusedName(name, nbt) => format!("unused {nbt}: `{name}`"),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            HirWarningKind::RedefPrelude(_) => String::from("It's okay to do so, but it might confuse you."),
            _ => String::new(),
        }
    }
}
