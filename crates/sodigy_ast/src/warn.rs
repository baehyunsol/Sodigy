use crate::IdentWithSpan;
use smallvec::{smallvec, SmallVec};
use sodigy_error::{ExtraErrInfo, SodigyError, SodigyErrorKind};
use sodigy_intern::InternSession;
use sodigy_parse::Punct;
use sodigy_span::SpanRange;

pub struct AstWarning {
    kind: AstWarningKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrInfo,
}

impl AstWarning {
    pub fn ambiguous_type_in_pattern(punct: Punct, span: SpanRange) -> Self {
        AstWarning {
            kind: AstWarningKind::AmbiguousTypeInPattern(punct),
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn multiple_bindings_on_one_pattern(bind1: IdentWithSpan, bind2: IdentWithSpan) -> Self {
        AstWarning {
            kind: AstWarningKind::MultipleBindingsOnOnePattern,
            spans: smallvec![*bind1.span(), *bind2.span()],
            extra: ExtraErrInfo::none(),
        }
    }
}

impl SodigyError<AstWarningKind> for AstWarning {
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

    fn err_kind(&self) -> &AstWarningKind {
        &self.kind
    }

    fn is_warning(&self) -> bool {
        true
    }

    fn index(&self) -> u32 {
        1
    }
}

pub enum AstWarningKind {
    AmbiguousTypeInPattern(Punct),
    MultipleBindingsOnOnePattern,
}

impl SodigyErrorKind for AstWarningKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            AstWarningKind::AmbiguousTypeInPattern(op) => format!("ambiguous operator `{op}` in type annotation"),
            AstWarningKind::MultipleBindingsOnOnePattern => String::from("multiple name bindings on a single pattern"),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            // TODO: how do I silence this warning if the user really mean this?
            AstWarningKind::AmbiguousTypeInPattern(op) => format!("It's very likely that you meant to use `{op}` in a pattern, but it's inside a type annotation. Use parenthesis to remove ambiguity."),
            AstWarningKind::MultipleBindingsOnOnePattern => String::from("There's no point in binding multiple names on a pattern."),
        }
    }

    fn index(&self) -> u32 {
        match self {
            AstWarningKind::AmbiguousTypeInPattern(_) => 0,
            AstWarningKind::MultipleBindingsOnOnePattern => 1,
        }
    }
}
