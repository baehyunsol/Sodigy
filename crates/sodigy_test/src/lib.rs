pub const TEST_MODE: bool = true;

// TODO: I don't know how to use `*`s in `macro_rules`

#[macro_export]
macro_rules! sodigy_assert {
    ($val: expr) => { assert!($val); };
    ($val: expr, ) => { assert!($val); };
    ($_: expr) => { (); };
}

#[macro_export]
macro_rules! sodigy_assert_eq {
    ($val1: expr, $val2: expr) => { assert_eq!($val1, $val2); };
    ($val1: expr, $val2: expr, ) => { assert_eq!($val1, $val2); };
    ($_v1: expr, $_v2: expr) => { (); };
}
