use std::num::NonZero;

#[derive(Clone, Debug)]
pub enum Error {
    NonZeroExit(NonZero<u8>),
    TestFail,
    CannotFindEntry,
}
