use sodigy_hir as hir;

mod block;
mod expr;
mod func;
mod r#if;
mod r#let;
mod session;
mod r#type;

pub use block::Block;
pub use expr::{Callable, Expr};
pub use func::Func;
pub use r#if::If;
pub use r#let::Let;
pub use session::Session;
pub use r#type::Type;

pub fn lower(hir_session: &hir::Session) -> Session {
    let mut session = Session::from_hir_session(hir_session);

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

    session
}
