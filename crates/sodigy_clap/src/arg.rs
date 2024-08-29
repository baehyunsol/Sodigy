use crate::DumpType;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Arg {
    Path(String),
    String(String),
    Integer(i64),
    Library(HashMap<String, String>),
    DumpType(DumpType),
}

impl Arg {
    pub fn unwrap_int(&self) -> i64 {
        match self {
            Arg::Integer(n) => *n,
            _ => panic!(),
        }
    }

    pub fn unwrap_string(&self) -> String {
        match self {
            Arg::String(s) => s.to_string(),
            _ => panic!(),
        }
    }

    pub fn unwrap_path(&self) -> String {
        match self {
            Arg::Path(p) => p.to_string(),
            _ => panic!(),
        }
    }

    pub fn unwrap_library(&self) -> HashMap<String, String> {
        match self {
            Arg::Library(l) => l.clone(),
            _ => panic!(),
        }
    }

    pub fn unwrap_dump_type(&self) -> DumpType {
        match self {
            Arg::DumpType(d) => *d,
            _ => panic!(),
        }
    }
}
