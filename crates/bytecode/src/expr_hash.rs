use crate::{Label, Memory, SSA, Value};
use sodigy_endec::Endec;
use sodigy_mir::Intrinsic;
use sodigy_utils::{dump_hex, hash};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExprHash(pub(crate) u128);

impl ExprHash {
    pub fn from_const(c: &Value) -> ExprHash {
        let mut encoded = vec![0];
        c.encode_impl(&mut encoded);
        ExprHash(hash(&encoded))
    }

    pub fn from_func_call(f: &Label, args: &[SSA]) -> ExprHash {
        let mut encoded = vec![1];
        f.encode_impl(&mut encoded);

        for arg in args.iter() {
            arg.encode_impl(&mut encoded);
        }

        ExprHash(hash(&encoded))
    }

    pub fn from_dynamic_func_call(f: &Memory, args: &[SSA]) -> ExprHash {
        let mut encoded = vec![2];
        f.encode_impl(&mut encoded);

        for arg in args.iter() {
            arg.encode_impl(&mut encoded);
        }

        ExprHash(hash(&encoded))
    }

    pub fn from_intrinsic(f: Intrinsic, args: &[SSA]) -> ExprHash {
        let mut encoded = vec![3];
        f.encode_impl(&mut encoded);

        for arg in args.iter() {
            arg.encode_impl(&mut encoded);
        }

        ExprHash(hash(&encoded))
    }

    pub fn hex(&self, l: usize) -> String {
        dump_hex(self.0, l)
    }
}
