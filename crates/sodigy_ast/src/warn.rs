use crate::IdentWithSpan;
use smallvec::{smallvec, SmallVec};
use sodigy_error::{ExtraErrorInfo, SodigyError, SodigyErrorKind, Stage};
use sodigy_intern::InternSession;
use sodigy_span::SpanRange;

pub struct AstWarning {
    kind: AstWarningKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrorInfo,
}

impl AstWarning {
    pub fn multiple_bindings_on_one_pattern(bind1: IdentWithSpan, bind2: IdentWithSpan) -> Self {
        AstWarning {
            kind: AstWarningKind::MultipleBindingsOnOnePattern,
            spans: smallvec![*bind1.span(), *bind2.span()],
            extra: ExtraErrorInfo::none(),
        }
    }
}

impl SodigyError<AstWarningKind> for AstWarning {
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

    fn error_kind(&self) -> &AstWarningKind {
        &self.kind
    }

    fn is_warning(&self) -> bool {
        true
    }

    fn index(&self) -> u32 {
        1
    }

    fn get_stage(&self) -> Stage {
        Stage::Ast
    }
}

pub enum AstWarningKind {
    MultipleBindingsOnOnePattern,
}

impl SodigyErrorKind for AstWarningKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            AstWarningKind::MultipleBindingsOnOnePattern => String::from("multiple name bindings on a pattern"),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            AstWarningKind::MultipleBindingsOnOnePattern => String::from("There's no point in binding multiple names on a pattern."),
        }
    }

    fn index(&self) -> u32 {
        match self {
            AstWarningKind::MultipleBindingsOnOnePattern => 0,
        }
    }
}
