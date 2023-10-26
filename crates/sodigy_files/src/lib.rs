mod cache;
mod err;
mod funcs;
mod session;

use std::sync::Mutex;

pub use err::*;
pub use funcs::*;
pub use session::{
    FileHash,
    Session as FileSession,
};

pub const DUMMY_FILE_HASH: FileHash = 0;

pub unsafe fn global_file_session() -> &'static mut FileSession {
    if !IS_INIT {
        init_global_file_session();
    }

    GLOBAL_FILE_SESSION.as_mut().unwrap()
}

unsafe fn init_global_file_session() {
    if IS_INIT {
        return;
    }

    let lock = LOCK.lock();
    let mut g = Box::new(FileSession::new());
    GLOBAL_FILE_SESSION = g.as_mut() as *mut _;
    IS_INIT = true;
    drop(lock);
    std::mem::forget(g);
}

static mut IS_INIT: bool = false;
static mut GLOBAL_FILE_SESSION: *mut FileSession = std::ptr::null_mut();
pub(crate) static mut LOCK: Mutex<()> = Mutex::new(());
