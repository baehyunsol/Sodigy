// NOTE: these 2 are copy-paste of each other
mod mut_;
mod immut;

pub use mut_::{mut_walker_func, mut_walker_expr};
pub use immut::{walker_func, walker_expr};

pub struct EmptyWalkerState;
