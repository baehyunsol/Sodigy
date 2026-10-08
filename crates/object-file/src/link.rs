use crate::{Assert, Entry, ObjectFile};
use sodigy_bytecode::GlobalLabel;
use std::collections::hash_map::{Entry as HashMapEntry, HashMap};

pub fn link(object_files: Vec<ObjectFile>) -> ObjectFile {
    let mut data = HashMap::new();
    let mut code = vec![];
    let mut entry_point: Option<GlobalLabel> = None;
    let mut asserts: Vec<Assert> = vec![];
    let mut no_entry = false;

    for mut object_file in object_files.into_iter() {
        for (key, value) in object_file.data.drain() {
            if let HashMapEntry::Vacant(e) = data.entry(key) {
                e.insert(value);
            }
        }

        match (object_file.entry, &entry_point) {
            (Entry::Main(e1), Some(e2)) => {
                assert_eq!(e1, *e2);
            },
            (Entry::Main(e), None) => {
                entry_point = Some(e);
            },
            (Entry::Asserts(a), _) => {
                asserts.extend(a);
            },
            (Entry::NoEntry, _) => {
                no_entry = true;
            },
        }

        code.extend(object_file.code.drain());
    }

    let entry = match entry_point {
        Some(e) => {
            assert!(asserts.is_empty());
            assert!(!no_entry);
            Entry::Main(e)
        },
        _ if no_entry => {
            assert!(asserts.is_empty());
            Entry::NoEntry
        },
        _ => {
            assert!(!no_entry);
            Entry::Asserts(asserts)
        },
    };

    ObjectFile {
        data,
        code: code.into_iter().collect(),
        entry,
    }
}

