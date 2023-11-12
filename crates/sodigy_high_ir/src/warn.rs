use crate::names::NameBindingType;
use smallvec::{smallvec, SmallVec};
use sodigy_ast::{self as ast, IdentWithSpan};
use sodigy_err::{ExtraErrInfo, SodigyError, SodigyErrorKind};
use sodigy_intern::{InternedString, InternSession};
use sodigy_span::SpanRange;

pub struct HirWarning {
    kind: HirWarningKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrInfo,
}

impl HirWarning {
    pub fn redef_prelude(id: IdentWithSpan) -> Self {
        HirWarning {
            kind: HirWarningKind::RedefPrelude(*id.id()),
            spans: smallvec![*id.span()],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn unused_name(id: IdentWithSpan, binding_type: NameBindingType) -> Self {
        HirWarning {
            kind: HirWarningKind::UnusedName(*id.id(), binding_type),
            spans: smallvec![*id.span()],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn unnecessary_paren(e: &ast::Expr) -> Self {
        HirWarning {
            kind: HirWarningKind::UnnecessaryParen {
                is_brace: matches!(&e.kind, ast::ExprKind::Value(ast::ValueKind::Scope { .. })),
            },
            spans: smallvec![e.span.first_char(), e.span.last_char()],
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
    UnnecessaryParen {
        is_brace: bool,
    },
}

impl SodigyErrorKind for HirWarningKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            HirWarningKind::RedefPrelude(name) => format!("redefinition of prelude `{name}`"),
            HirWarningKind::UnusedName(name, nbt) => format!("unused {}: `{name}`", nbt.render_error()),
            HirWarningKind::UnnecessaryParen { .. } => format!("unnecessary parenthesis"),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            HirWarningKind::RedefPrelude(_) => String::from("It's okay to do so, but it might confuse you."),
            HirWarningKind::UnnecessaryParen { is_brace } => if *is_brace {
                String::from("This curly brace doesn't do anything.")
            } else {
                String::from("This parenthesis doesn't do anything.")
            },
            _ => String::new(),
        }
    }
}
