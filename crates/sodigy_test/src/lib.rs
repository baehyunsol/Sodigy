#![deny(unused_imports)]

mod log;

// set it to false when you want to disable the test statements
pub const TEST_MODE: bool = true;
pub use log::{
    sodigy_log,
    LOG_FILE_PATH,
    LOG_IMPORTANT,
    LOG_NORMAL,
    LOG_VERBOSE,
};

// TODO: rust std has `debug_assert` and `debug_assert_eq` -> test them, if they do well, the entire `sodigy_test` can be removed
// TODO: How does `$(..),*` syntax work?

// choose empty branches when you want to disable asserts in the code
#[macro_export]
macro_rules! sodigy_assert {
    ($($x: expr),* $(,)?) => { assert!($($x),*); };
    ($($_x: expr),* $(,)?) => { (); };
}

#[macro_export]
macro_rules! sodigy_assert_eq {
    ($($x: expr),* $(,)?) => { assert_eq!($($x),*); };
    ($($_x: expr),* $(,)?) => { (); };
}

// `sodigy_log!` is for debugging the compiler. it can be disabled easily
// `sodigy_log_ice!` is for users to report internal compiler errors. it cannot be disabled

#[macro_export]
macro_rules! sodigy_log {
    ($level: expr, $msg: expr $(,)?) => { sodigy_log($level, $msg); };
    ($_x: expr, $_y: expr $(,)?) => { (); };
}

#[macro_export]
macro_rules! sodigy_log_ice {
    ($msg: expr $(,)?) => { sodigy_log(LOG_IMPORTANT, $msg) };
}
