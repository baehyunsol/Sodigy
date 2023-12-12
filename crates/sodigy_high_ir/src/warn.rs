use crate::names::NameBindingType;
use crate::pattern::{NumberLike, RangeType};
use smallvec::{smallvec, SmallVec};
use sodigy_ast::{self as ast, IdentWithSpan};
use sodigy_error::{ExtraErrInfo, RenderError, SodigyError, SodigyErrorKind};
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
            kind: HirWarningKind::RedefPrelude(id.id()),
            spans: smallvec![*id.span()],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn unused_name(id: IdentWithSpan, binding_type: NameBindingType) -> Self {
        HirWarning {
            kind: HirWarningKind::UnusedName(id.id(), binding_type),
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

    pub fn point_range(from: NumberLike, to: NumberLike, ty: RangeType, span: SpanRange) -> Self {
        HirWarning {
            kind: HirWarningKind::PointRange { from, to, ty },
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn name_binding_on_wildcard(bind: IdentWithSpan) -> Self {
        HirWarning {
            kind: HirWarningKind::NameBindingOnWildcard,
            spans: smallvec![*bind.span()],
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

    fn index(&self) -> u32 {
        6
    }
}

pub enum HirWarningKind {
    RedefPrelude(InternedString),
    UnusedName(InternedString, NameBindingType),
    UnnecessaryParen {
        is_brace: bool,
    },
    PointRange {  // `0..~0`
        from:NumberLike,
        to: NumberLike,
        ty: RangeType,
    },
    NameBindingOnWildcard,
}

impl SodigyErrorKind for HirWarningKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            HirWarningKind::RedefPrelude(name) => format!("redefinition of prelude `{}`", name.render_error()),
            HirWarningKind::UnusedName(name, nbt) => format!("unused {}: `{}`", nbt.render_error(), name.render_error()),
            HirWarningKind::UnnecessaryParen { .. } => format!("unnecessary parenthesis"),
            HirWarningKind::PointRange { .. } => format!("meaningless range"),
            HirWarningKind::NameBindingOnWildcard => format!("name binding on wildcard"),
        }
    }

    fn help(&self, session: &mut InternSession) -> String {
        match self {
            HirWarningKind::RedefPrelude(_) => String::from("It's okay to do so, but it might confuse you."),
            HirWarningKind::UnnecessaryParen { is_brace } => if *is_brace {
                String::from("This curly brace doesn't do anything.")
            } else {
                String::from("This parenthesis doesn't do anything.")
            },
            HirWarningKind::PointRange { from, ty, .. } => {
                let rendered = match ty {
                    RangeType::Char => format!(
                        "{:?}",
                        char::from_u32(from.try_into_u32(session).unwrap()).unwrap(),
                    ),
                    _ => from.render_error(),
                };

                format!("`{rendered}..~{rendered}` is just `{rendered}`.")
            },
            HirWarningKind::NameBindingOnWildcard => String::from("This name binding doesn't do anything."),
            _ => String::new(),
        }
    }

    fn index(&self) -> u32 {
        match self {
            HirWarningKind::RedefPrelude(..) => 0,
            HirWarningKind::UnusedName(..) => 1,
            HirWarningKind::UnnecessaryParen { .. } => 2,
            HirWarningKind::PointRange { .. } => 3,
            HirWarningKind::NameBindingOnWildcard => 4,
        }
    }
}
