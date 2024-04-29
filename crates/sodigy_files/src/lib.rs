#![deny(unused_imports)]
#![feature(io_error_more)]

mod cache;
mod error;
mod funcs;
mod session;

use std::sync::Mutex;

pub use error::*;
pub use funcs::*;
pub use session::{
    FileHash,
    FileSession,
};

pub const DUMMY_FILE_HASH: FileHash = 0;

pub unsafe fn global_file_session() -> &'static mut FileSession {
    if !IS_FILE_SESSION_INIT {
        init_global_file_session();
    }

    GLOBAL_FILE_SESSION.as_mut().unwrap()
}

unsafe fn init_global_file_session() {
    if IS_FILE_SESSION_INIT {
        return;
    }

    let lock = LOCK.lock().unwrap();

    // see comments in sodigy_intern::global::init_global
    if IS_FILE_SESSION_INIT {
        return;
    }

    let mut g = Box::new(FileSession::new());
    GLOBAL_FILE_SESSION = g.as_mut() as *mut _;
    IS_FILE_SESSION_INIT = true;
    drop(lock);
    std::mem::forget(g);
}

pub(crate) static mut IS_FILE_SESSION_INIT: bool = false;
static mut GLOBAL_FILE_SESSION: *mut FileSession = std::ptr::null_mut();
pub(crate) static mut LOCK: Mutex<()> = Mutex::new(());
