#![deny(unused_imports)]

// set it to false when you want to disable the test statements
pub const TEST_MODE: bool = true;

// TODO: I'm not sure with `,*` and `$(,)?` syntax

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
