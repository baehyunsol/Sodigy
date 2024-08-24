use smallvec::SmallVec;
use std::fmt;

#[derive(Clone)]
pub enum JsonObj {
    Array(Vec<JsonObj>),
    Bool(bool),
    Int(i64),
    Table(Vec<(String, JsonObj)>),
    String(String),
    Null,
}

impl JsonObj {
    pub fn push_pair(&mut self, key: &str, value: JsonObj) -> Result<(), ()> {
        match self {
            JsonObj::Table(v) => {
                v.push((key.to_string(), value));
                Ok(())
            },
            _ => Err(()),
        }
    }

    pub fn pretty(&self, indent: u16) -> String {
        json::parse(&self.to_string()).unwrap().pretty(indent)
    }
}

pub trait DumpJson {
    fn dump_json(&self) -> JsonObj;
}

impl DumpJson for JsonObj {
    fn dump_json(&self) -> JsonObj {
        self.clone()
    }
}

impl DumpJson for String {
    fn dump_json(&self) -> JsonObj {
        JsonObj::String(self.to_string())
    }
}

impl DumpJson for &str {
    fn dump_json(&self) -> JsonObj {
        JsonObj::String(self.to_string())
    }
}

impl DumpJson for bool {
    fn dump_json(&self) -> JsonObj {
        JsonObj::Bool(*self)
    }
}

impl<T: DumpJson> DumpJson for Option<T> {
    fn dump_json(&self) -> JsonObj {
        match self {
            Some(v) => v.dump_json(),
            None => JsonObj::Null,
        }
    }
}

impl <T: DumpJson> DumpJson for Box<T> {
    fn dump_json(&self) -> JsonObj {
        self.as_ref().dump_json()
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
    /// It emits an evaluable json object. The result is not prettified, so you might
    /// need another tool that prettifies json.
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            JsonObj::Array(objects) => write!(
                fmt, "[{}]",
                objects.iter().map(|obj| obj.to_string()).collect::<Vec<_>>().join(",")
            ),
            JsonObj::Bool(b) => write!(
                fmt, "{b}",
            ),
            JsonObj::Int(i) => write!(
                fmt, "{i}",
            ),
            JsonObj::Table(objects) => write!(
                fmt, "{{{}}}",
                objects.iter().map(
                    |(k, v)| format!("{k:?}:{v}")
                ).collect::<Vec<_>>().join(","),
            ),
            JsonObj::String(s) => write!(
                fmt, "{s:?}",
            ),
            JsonObj::Null => write!(fmt, "null"),
        }
    }
}
