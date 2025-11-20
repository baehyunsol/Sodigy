use crate::{FileOrStd, ModulePath};

// TODO: It's just a proof-of-concept!
// TODO: I'm too lazy to calc its hash... so the hash values are all dummies
pub(crate) const STD_FILES: [(
    &'static str,   // module_path
    &'static str,   // file_path
    &'static [u8],  // contents
    u128,           // content_hash
); 11] = [
    (
        "@std.lib.bool",
        "@std/bool.sdg",
        include_bytes!("../../../std/bool.sdg"),
        1000,
    ),
    (
        "@std.lib.byte",
        "@std/byte.sdg",
        include_bytes!("../../../std/byte.sdg"),
        1001,
    ),
    (
        "@std.lib.bytes",
        "@std/bytes.sdg",
        include_bytes!("../../../std/bytes.sdg"),
        1002,
    ),
    (
        "@std.lib.char",
        "@std/char.sdg",
        include_bytes!("../../../std/char.sdg"),
        1003,
    ),
    (
        "@std.lib.int",
        "@std/int.sdg",
        include_bytes!("../../../std/int.sdg"),
        1004,
    ),
    (
        "@std.lib",
        "@std/lib.sdg",
        include_bytes!("../../../std/lib.sdg"),
        1005,
    ),
    (
        "@std.lib.list",
        "@std/list.sdg",
        include_bytes!("../../../std/list.sdg"),
        1006,
    ),
    (
        "@std.lib.number",
        "@std/number.sdg",
        include_bytes!("../../../std/number.sdg"),
        1007,
    ),
    (
        "@std.lib.op",
        "@std/op.sdg",
        include_bytes!("../../../std/op.sdg"),
        1008,
    ),
    (
        "@std.lib.prelude",
        "@std/prelude.sdg",
        include_bytes!("../../../std/prelude.sdg"),
        1009,
    ),
    (
        "@std.lib.string",
        "@std/string.sdg",
        include_bytes!("../../../std/string.sdg"),
        1010,
    ),
];

pub fn std_root() -> (ModulePath, FileOrStd) {
    (
        ModulePath {
            path: vec![],
            is_std: true,
        },
        FileOrStd::Std(5),
    )
}
