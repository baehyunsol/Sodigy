mod fmt;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Uid(u128);

// first 4 bits: Type of Uid

// next 4 bits: Metadata
//     is_prelude, XX, XX, XX

// last 120 bits: Random Index
// It assumes that the 120 bit random bit sequence is always unique.

const ERASER: u128 = 0x00ff_ffff_ffff_ffff_ffff_ffff_ffff_ffff;

// Uid types
pub(crate) const DEF: u128 = 0x0 << 124;
pub(crate) const ENUM: u128 = 0x1 << 124;
pub(crate) const STRUCT: u128 = 0x2 << 124;
pub(crate) const MODULE: u128 = 0x3 << 124;
pub(crate) const LAMBDA: u128 = 0x4 << 124;
pub(crate) const SCOPE_BLOCK: u128 = 0x5 << 124;
pub(crate) const MATCH_ARM: u128 = 0x6 << 124;

// Metadata
pub(crate) const PRELUDE_MASK: u128 = 0b1000 << 120;

impl Uid {
    pub fn new_scope() -> Self {
        Uid(rand::random::<u128>() & ERASER | SCOPE_BLOCK)
    }

    pub fn new_def() -> Self {
        Uid(rand::random::<u128>() & ERASER | DEF)
    }

    pub fn new_enum() -> Self {
        Uid(rand::random::<u128>() & ERASER | ENUM)
    }

    pub fn new_struct() -> Self {
        Uid(rand::random::<u128>() & ERASER | STRUCT)
    }

    pub fn new_module() -> Self {
        Uid(rand::random::<u128>() & ERASER | MODULE)
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

    pub fn is_prelude(self) -> bool {
        self.0 & PRELUDE_MASK != 0
    }

    // result < 16
    pub fn get_type(self) -> u32 {
        (self.0 >> 124) as u32
    }

    // result < 16
    pub fn get_metadata(self) -> u32 {
        ((self.0 & (0xf << 120)) >> 120) as u32
    }
}
