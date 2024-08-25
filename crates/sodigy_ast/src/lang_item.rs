use crate::expr::{Expr, ExprKind};
use crate::value::ValueKind;
use sodigy_intern::InternSession;
use sodigy_lang_item::LangItem;
use sodigy_span::SpanRange;

pub fn create_lang_item(
    item_name: LangItem,
    span: SpanRange,
    intern_session: &mut InternSession,
) -> Expr {
    Expr {
        kind: ExprKind::Value(ValueKind::Identifier(item_name.into_interned_string(intern_session))),
        span,
    }
}
