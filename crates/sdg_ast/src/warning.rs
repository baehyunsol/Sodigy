use crate::err::ParamType;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;

mod kind;

pub use kind::SodigyWarningKind;

pub struct SodigyWarning {
    kind: SodigyWarningKind,
    pub(crate) span: Span,
    message: String,
}

impl SodigyWarning {
    pub fn unused_param(name: InternedString, span: Span, param_type: ParamType) -> Self {
        SodigyWarning {
            kind: SodigyWarningKind::UnusedParam(name, param_type),
            span,
            message: String::new(),
        }
    }

    pub fn unused_use(name: InternedString, span: Span) -> Self {
        SodigyWarning {
            kind: SodigyWarningKind::UnusedUse(name),
            span,
            message: String::new(),
        }
    }

    pub fn render_warning(&self, session: &LocalParseSession) -> String {
        format!(
            "Warning: {}{}{}",
            self.kind.render_warning(session),
            if self.message.len() > 0 {
                format!("\n{}", self.message)
            } else {
                String::new()
            },
            if self.span.is_dummy() {
                String::new()
            } else {
                format!("\n{}", self.span.render_err(session))
            }
        )
    }
}
