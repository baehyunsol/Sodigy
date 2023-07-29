mod global;
mod interned_string;
mod local;

pub use global::{GlobalParseSession, GLOBAL_SESSION, GLOBAL_SESSION_LOCK, KEYWORDS, try_init_global_session};
pub use interned_string::{InternedString, KEYWORD_START};
pub use local::LocalParseSession;

pub const DUMMY_FILE_INDEX: u64 = u64::MAX;