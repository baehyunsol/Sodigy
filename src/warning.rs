use crate::err::ParamType;
use crate::session::InternedString;
use crate::span::Span;

mod kind;

pub use kind::SodigyWarningKind;

pub struct SodigyWarning {
    kind: SodigyWarningKind,
    span: Span,
    message: String,
}

impl SodigyWarning {
    pub fn unused(name: InternedString, span: Span, param_type: ParamType) -> Self {
        SodigyWarning {
            kind: SodigyWarningKind::UnusedParam(name, param_type),
            span,
            message: String::new(),
        }
    }
}