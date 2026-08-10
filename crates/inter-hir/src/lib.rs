// It's defined here because it defines a macro.
mod log;

mod alias;
mod assoc;
mod endec;
mod expr;
mod module;
mod path;
mod pattern;
mod poly;
mod session;
mod r#type;

pub use assoc::get_associated_func_name;
use expr::{TypeStructExpr, not_x_but_y};
pub use log::{LogEntry, LogId};
use log::write_log;
pub use session::Session;
