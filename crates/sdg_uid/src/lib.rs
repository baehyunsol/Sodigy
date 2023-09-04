#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct UID(pub(crate) u128);

// TODO: make everything const if it has no randomness

pub mod prelude;

macro_rules! def_new_uid {
    ($method_name: ident, $const_name: ident) => {
        impl UID {
            pub fn $method_name() -> Self {
                UID(rand::random::<u128>() & ZERO | $const_name)
            }
        }
    }
}

impl UID {
    pub fn dummy() -> Self {
        UID(DUMMY)
    }

    pub fn is_dummy(&self) -> bool {
        self.0 == DUMMY
    }
}

def_new_uid!(new_block_id, BLOCK);
def_new_uid!(new_func_id, FUNC);
def_new_uid!(new_enum_id, ENUM);
def_new_uid!(new_enum_var_id, ENUM_VAR);
def_new_uid!(new_struct_id, STRUCT);
def_new_uid!(new_lambda_id, LAMBDA);
def_new_uid!(new_match_id, MATCH);
def_new_uid!(new_match_branch_id, MATCH_BRANCH);

impl UID {
    pub fn to_string(&self) -> String {
        format!("{:x}", self.0)
    }

    pub fn to_u128(&self) -> u128 {
        self.0
    }
}

pub(crate) const DUMMY: u128 = 0;

pub(crate) const ZERO: u128 = 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_F000;
pub(crate) const BLOCK: u128  = 0x001;
pub(crate) const FUNC: u128   = 0x002;
pub(crate) const ENUM: u128   = 0x003;
pub(crate) const ENUM_VAR: u128   = 0x004;
pub(crate) const STRUCT: u128 = 0x005;
pub(crate) const LAMBDA: u128 = 0x006;
pub(crate) const MATCH: u128  = 0x007;
pub(crate) const MATCH_BRANCH: u128 = 0x008;
pub(crate) const PRELUDE: u128 = 0x009;
