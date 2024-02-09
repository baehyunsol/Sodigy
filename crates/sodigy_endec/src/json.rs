use smallvec::SmallVec;
use std::fmt;

pub enum JsonObj {
    Array(Vec<JsonObj>),
    Bool(bool),
    Int(i64),
    Table(Vec<(String, JsonObj)>),
    String(String),
}

pub trait DumpJson {
    fn dump_json(&self) -> JsonObj;
}

impl<T: DumpJson> DumpJson for &T {
    fn dump_json(&self) -> JsonObj {
        self.dump_json()
    }
}

impl DumpJson for String {
    fn dump_json(&self) -> JsonObj {
        JsonObj::String(self.to_string())
    }
}

impl DumpJson for bool {
    fn dump_json(&self) -> JsonObj {
        JsonObj::Bool(*self)
    }
}

macro_rules! simple_impl_for_dump_json {
    (int, $t: ty) => {
        impl DumpJson for $t {
            fn dump_json(&self) -> JsonObj {
                JsonObj::Int(*self as i64)
            }
        }
    };
    (vec, $t: ident, $u: ty) => {
        impl<$t: DumpJson> DumpJson for $u {
            fn dump_json(&self) -> JsonObj {
                JsonObj::Array(self.iter().map(|e| e.dump_json()).collect::<Vec<JsonObj>>())
            }
        }
    };
}

simple_impl_for_dump_json!(int, u8);
simple_impl_for_dump_json!(int, u16);
simple_impl_for_dump_json!(int, u32);
simple_impl_for_dump_json!(int, u64);
simple_impl_for_dump_json!(int, usize);
simple_impl_for_dump_json!(int, u128);
simple_impl_for_dump_json!(int, i8);
simple_impl_for_dump_json!(int, i16);
simple_impl_for_dump_json!(int, i32);
simple_impl_for_dump_json!(int, i64);
simple_impl_for_dump_json!(int, isize);
simple_impl_for_dump_json!(int, i128);

simple_impl_for_dump_json!(vec, T, Vec<T>);
simple_impl_for_dump_json!(vec, T, SmallVec<[T; 1]>);

pub fn json_key_value_table(items: Vec<(&str, JsonObj)>) -> JsonObj {
    JsonObj::Table(items.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

impl fmt::Display for JsonObj {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        todo!()
    }
}
