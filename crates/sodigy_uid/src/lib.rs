#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Uid(u128);

// first 6 bits: Type of Uid

// next 10 bits: Metadata
//     is_prelude, XX, XX, XX, XX,
//             XX, XX, XX, XX, XX,

// last 112 bits: Random Index
// It assumes that 112 bit random bit sequence is always unique.

const ERASER: u128 = 0x0000_ffff_ffff_ffff_ffff_ffff_ffff_ffff;

// Uid types
const DEF: u128 = 0b_000_001 << 122;
const LAMBDA: u128 = 0b_000_010 << 122;
const SCOPE_BLOCK: u128 = 0b_000_011 << 122;
const MATCH_ARM: u128 = 0b_000_100 << 122;

// Metadata
const PRELUDE_MASK: u128 = 1 << 121;

impl Uid {
    pub fn new_scope() -> Self {
        Uid(rand::random::<u128>() & ERASER | SCOPE_BLOCK)
    }

    pub fn new_def() -> Self {
        Uid(rand::random::<u128>() & ERASER | DEF)
    }

    pub fn new_lambda() -> Self {
        Uid(rand::random::<u128>() & ERASER | LAMBDA)
    }

    pub fn new_match_arm() -> Self {
        Uid(rand::random::<u128>() & ERASER | MATCH_ARM)
    }

    #[must_use = "method returns a new uid and does not mutate the original value"]
    pub fn mark_prelude(self) -> Self {
        Uid(self.0 | PRELUDE_MASK)
    }
}
