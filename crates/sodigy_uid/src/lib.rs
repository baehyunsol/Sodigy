#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Uid(u128);

// first 8 bits: Type of Uid
// next 8 bits: Metadata
// last 112 bits: Random Index
// It assumes that 112 bit random bit sequence is always unique.

const ERASER: u128 = 0x0000_ffff_ffff_ffff_ffff_ffff_ffff_ffff;

// Uid types
const FUNC_DEF: u128 = 0x01 << 120;
const LAMBDA: u128 = 0x02 << 120;
const SCOPE_BLOCK: u128 = 0x03 << 120;
const MATCH_ARM: u128 = 0x04 << 120;

impl Uid {
    pub fn new_scope() -> Self {
        Uid(rand::random::<u128>() & ERASER | SCOPE_BLOCK)
    }

    pub fn new_func() -> Self {
        Uid(rand::random::<u128>() & ERASER | FUNC_DEF)
    }

    pub fn new_lambda() -> Self {
        Uid(rand::random::<u128>() & ERASER | LAMBDA)
    }

    pub fn new_match_arm() -> Self {
        Uid(rand::random::<u128>() & ERASER | MATCH_ARM)
    }
}
