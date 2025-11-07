use crate::{FileOrStd, ModulePath};

// TODO: It's just a proof-of-concept!
// TODO: I'm too lazy to calc its hash... so the hash values are all dummies
pub(crate) const STD_FILES: [(
    &'static str,   // module_path
    &'static str,   // file_path
    &'static [u8],  // contents
    u128,           // content_hash
); 5] = [
    (
        "@std.lib",
        "@std/lib.sdg",
        include_bytes!("../../../std/lib.sdg"),
        1000,
    ), (
        "@std.lib.built_in",
        "@std/built_in.sdg",
        include_bytes!("../../../std/built_in.sdg"),
        1001,
    ), (
        "@std.lib.built_in.fns",
        "@std/built_in/fns.sdg",
        include_bytes!("../../../std/built_in/fns.sdg"),
        1002,
    ), (
        "@std.lib.built_in.traits",
        "@std/built_in/traits.sdg",
        include_bytes!("../../../std/built_in/traits.sdg"),
        1003,
    ), (
        "@std.lib.built_in.types",
        "@std/built_in/types.sdg",
        include_bytes!("../../../std/built_in/types.sdg"),
        1004,
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
