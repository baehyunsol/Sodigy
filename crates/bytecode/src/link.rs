use crate::ObjectFile;
use std::collections::hash_map::{Entry, HashMap};

pub fn link(object_files: Vec<ObjectFile>) -> ObjectFile {
    let mut data = HashMap::new();
    let mut code = vec![];
    let mut main_entry = None;
    let mut asserts = vec![];

    for mut object_file in object_files.into_iter() {
        for (key, value) in object_file.data.drain() {
            if let Entry::Vacant(e) = data.entry(key) {
                e.insert(value);
            }
        }

        // TODO: What if there are multiple object files that have a main entry?
        if let Some(main) = object_file.main_entry {
            main_entry = Some(main);
        }

        code.extend(object_file.code.drain());
        asserts.extend(object_file.asserts.drain(..));
    }

    ObjectFile {
        data,
        code: code.into_iter().collect(),
        main_entry,
        asserts,
    }
}
