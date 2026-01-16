use sodigy_hir::Session as HirSession;
use sodigy_inter_hir::Session as InterHirSession;

mod assert;
mod block;
pub(crate) mod dump;
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
pub(crate) use r#if::lower_hir_if;
pub use intrinsic::Intrinsic;
pub use r#let::Let;
pub use r#match::{Match, MatchArm};
pub use session::Session;
pub use r#struct::Struct;
pub use r#type::{Type, TypeAssertion, type_of};

pub fn lower(
    hir_session: HirSession,
    inter_hir_session: &InterHirSession,
) -> Session {
    let mut session = Session::from_hir(&hir_session, inter_hir_session);

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

    for hir_struct in hir_session.structs.iter() {
        if let Ok(r#struct) = Struct::from_hir(hir_struct, &mut session) {
            session.structs.push(r#struct);
        }
    }

    for type_assertion in hir_session.type_assertions.iter() {
        if let Ok(r#type) = Type::from_hir(&type_assertion.r#type, &mut session) {
            session.type_assertions.push(TypeAssertion {
                name_span: type_assertion.name_span,
                type_span: type_assertion.type_span,
                r#type,
            });
        }
    }

    session
}
