use crate::pattern::{NumberLike, RangeType};
use smallvec::{SmallVec, smallvec};
use sodigy_ast as ast;
use sodigy_error::{ExtraErrorInfo, RenderError, SodigyError, SodigyErrorKind, Stage};
use sodigy_intern::{InternedString, InternSession};
use sodigy_parse::IdentWithSpan;
use sodigy_span::SpanRange;

mod endec;

pub struct HirWarning {
    kind: HirWarningKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrorInfo,
}

impl HirWarning {
    pub fn redef_prelude(id: IdentWithSpan) -> Self {
        HirWarning {
            kind: HirWarningKind::RedefPrelude(id.id()),
            spans: smallvec![*id.span()],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn unnecessary_paren(e: &ast::Expr) -> Self {
        HirWarning {
            kind: HirWarningKind::UnnecessaryParen {
                is_brace: matches!(&e.kind, ast::ExprKind::Value(ast::ValueKind::Scope { .. })),
            },
            spans: smallvec![e.span.first_char(), e.span.last_char()],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn point_range(from: NumberLike, to: NumberLike, ty: RangeType, span: SpanRange) -> Self {
        HirWarning {
            kind: HirWarningKind::PointRange { from, to, ty },
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn name_binding_on_wildcard(bind: IdentWithSpan) -> Self {
        HirWarning {
            kind: HirWarningKind::NameBindingOnWildcard,
            spans: smallvec![*bind.span()],
            extra: ExtraErrorInfo::none(),
        }
    }
}

impl SodigyError<HirWarningKind> for HirWarning {
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

    fn error_kind(&self) -> &HirWarningKind {
        &self.kind
    }

    fn is_warning(&self) -> bool {
        true
    }

    fn index(&self) -> u32 {
        5
    }

    fn get_stage(&self) -> Stage {
        Stage::Hir
    }
}

pub enum HirWarningKind {
    RedefPrelude(InternedString),
    UnnecessaryParen {
        is_brace: bool,
    },
    PointRange {  // `0..~0`
        from: NumberLike,
        to: NumberLike,
        ty: RangeType,
    },
    NameBindingOnWildcard,
}

impl SodigyErrorKind for HirWarningKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            HirWarningKind::RedefPrelude(name) => format!("redefinition of prelude `{}`", name.render_error()),
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
        }
    }

    fn index(&self) -> u32 {
        match self {
            HirWarningKind::RedefPrelude(..) => 0,
            HirWarningKind::UnnecessaryParen { .. } => 1,
            HirWarningKind::PointRange { .. } => 2,
            HirWarningKind::NameBindingOnWildcard => 3,
        }
    }
}
