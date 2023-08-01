#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct UID(u128);

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
def_new_uid!(new_lambda_id, LAMBDA);
def_new_uid!(new_match_id, MATCH);
def_new_uid!(new_match_branch_id, MATCH_BRANCH);

impl UID {

    pub fn to_string(&self) -> String {
        format!("{:x}", self.0)
    }

}

const DUMMY: u128 = 0;

const ZERO: u128 = 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_F000;
const BLOCK: u128  = 0x001;
const FUNC: u128   = 0x002;
const LAMBDA: u128 = 0x003;
const MATCH: u128 = 0x004;
const MATCH_BRANCH: u128 = 0x005;
