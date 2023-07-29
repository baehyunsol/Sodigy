#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct UID(u128);

impl UID {

    pub fn new_block_id() -> Self {
        UID(rand::random::<u128>() & ZERO | BLOCK)
    }

    pub fn new_func_id() -> Self {
        UID(rand::random::<u128>() & ZERO | FUNC)
    }

    pub fn new_lambda_id() -> Self {
        UID(rand::random::<u128>() & ZERO | LAMBDA)
    }

    pub fn to_string(&self) -> String {
        format!("{:x}", self.0)
    }

}

const ZERO: u128 = 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_F000;
const BLOCK: u128  = 0x000;
const FUNC: u128   = 0x001;
const LAMBDA: u128 = 0x002;
