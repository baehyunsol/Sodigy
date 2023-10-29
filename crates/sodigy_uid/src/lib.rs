#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Uid(u128);

// first 8 bits: Type of Uid
// next 8 bits: Metadata
// last 112 bits: Random Index
// It assumes that 112 bit random bit sequence is always unique.

const ERASER: u128 = 0x0000_ffff_ffff_ffff_ffff_ffff_ffff_ffff;

// Uid types
const SCOPE_BLOCK: u128 = 0xff << 120;

impl Uid {
    pub fn new_scope() -> Self {
        Uid(rand::random::<u128>() & ERASER | SCOPE_BLOCK)
    }
}
