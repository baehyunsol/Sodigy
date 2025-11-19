use sodigy_hir::Session as HirSession;
use sodigy_inter_hir::Session as InterHirSession;

mod assert;
mod block;
mod endec;
mod r#enum;
mod expr;
mod func;
mod r#if;
mod intrinsic;
mod r#let;
mod r#match;
mod session;
mod r#struct;
mod r#type;

pub use assert::Assert;
pub use block::Block;
pub use r#enum::Enum;
pub use expr::{Callable, Expr, ShortCircuitKind};
pub use func::Func;
pub use r#if::If;
pub use intrinsic::Intrinsic;
pub use r#let::Let;
pub use r#match::Match;
pub use session::Session;
pub use r#struct::Struct;
pub use r#type::Type;

pub fn lower(
    hir_session: HirSession,
    inter_hir_session: &InterHirSession,
) -> Session {
    let mut session = Session::from_hir_session(&hir_session, inter_hir_session);

    for hir_let in hir_session.lets.iter() {
        if let Ok(r#let) = Let::from_hir(hir_let, &mut session) {
            session.lets.push(r#let);
        }
    }

    for hir_func in hir_session.funcs.iter() {
        if let Ok(func) = Func::from_hir(hir_func, &mut session) {
            session.funcs.push(func);
        }
    }

    for hir_assert in hir_session.asserts.iter() {
        if let Ok(assert) = Assert::from_hir(hir_assert, &mut session) {
            session.asserts.push(assert);
        }
    }

    session
}
