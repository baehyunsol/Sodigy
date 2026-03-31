use crate::{FileOrStd, ModulePath};
use sodigy_string::hash;
use std::collections::HashMap;
use std::sync::LazyLock;

pub struct StdFile {
    pub module_path: ModulePath,
    pub module_path_str: String,
    pub file_path: FileOrStd,
    pub file_path_str: String,
    pub contents: &'static [u8],
}

impl StdFile {
    pub fn file_hash(&self) -> u32 {
        let FileOrStd::Std(n) = &self.file_path else { unreachable!() };
        *n
    }
}

pub static STD_FILES: LazyLock<Vec<StdFile>> = LazyLock::new(|| {
    let mut result = vec![];
    let data: Vec<(&[&str], &[u8])> = vec![
        (&["bool"], include_bytes!("../../../std/bool.sdg")),
        (&["byte"], include_bytes!("../../../std/byte.sdg")),
        (&["bytes"], include_bytes!("../../../std/bytes.sdg")),
        (&["char"], include_bytes!("../../../std/char.sdg")),
        (&["convert"], include_bytes!("../../../std/convert.sdg")),
        (&["fn"], include_bytes!("../../../std/fn.sdg")),
        (&["int"], include_bytes!("../../../std/int.sdg")),
        (&["io"], include_bytes!("../../../std/io.sdg")),
        (&[], include_bytes!("../../../std/lib.sdg")),
        (&["list"], include_bytes!("../../../std/list.sdg")),
        (&["number"], include_bytes!("../../../std/number.sdg")),
        (&["op"], include_bytes!("../../../std/op.sdg")),
        (&["option"], include_bytes!("../../../std/option.sdg")),
        (&["prelude"], include_bytes!("../../../std/prelude.sdg")),
        (&["random"], include_bytes!("../../../std/random.sdg")),
        (&["result"], include_bytes!("../../../std/result.sdg")),
        (&["scalar"], include_bytes!("../../../std/scalar.sdg")),
        (&["string"], include_bytes!("../../../std/string.sdg")),
        (&["tuple"], include_bytes!("../../../std/tuple.sdg")),
    ];

    for (path, contents) in data.iter() {
        let h = (hash(contents) & 0x7fff_ffff) | 0x8000_0000;
        let module_path = ModulePath::init_std(path);
        let module_path_str = module_path.to_string();
        let file_path = FileOrStd::Std(h as u32);
        let mut path = path.to_vec();
        path.insert(0, "@std");

        if path.len() == 1 {
            path.push("lib");
        }

        let file_path_str = format!("{}.sdg", path.join("/"));
        result.push(StdFile {
            module_path,
            module_path_str,
            file_path,
            file_path_str,
            contents,
        });
    }

    result
});

pub static STD_FILE_INDEXES: LazyLock<HashMap<u32, usize>> = LazyLock::new(|| {
    let mut result = HashMap::with_capacity(STD_FILES.len());

    for (i, std_file) in STD_FILES.iter().enumerate() {
        if let Some(prev_i) = result.insert(std_file.file_hash(), i) {
            panic!(
                "There's a hash collision in std. This is a very unlucky case. The hash of {} and {} are the same. Edit either of the files and running it again will fix the issue.",
                STD_FILES[i].module_path_str,
                STD_FILES[prev_i].module_path_str,
            );
        }
    }

    result
});

pub static STD_ROOT_INDEX: LazyLock<usize> = LazyLock::new(|| {
    for (i, std_file) in STD_FILES.iter().enumerate() {
        if std_file.module_path_str == "@std.lib" {
            return i;
        }
    }

    unreachable!()
});

pub fn std_root() -> (ModulePath, FileOrStd) {
    use std::ops::Deref;
    let s = &STD_FILES[*STD_ROOT_INDEX.deref()];
    (s.module_path.clone(), s.file_path.clone())
}
