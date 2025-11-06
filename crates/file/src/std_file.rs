use crate::{FileOrStd, ModulePath};

// TODO: It's just a proof-of-concept!
pub(crate) const STD_FILES: [(
    &'static str,   // module_path
    &'static str,   // file_path
    &'static [u8],  // contents
    u128,           // content_hash
); 1] = [
    (
        "lib",
        "@std/lib.sdg",
        include_bytes!("../../../std/lib.sdg"),

        // TODO: I'm too lazy to calc its hash...
        1234,
    ),
];

pub fn std_root() -> (ModulePath, FileOrStd) {
    (
        ModulePath {
            path: vec![],
            is_std: true,
        },
        FileOrStd::Std(0),
    )
}
