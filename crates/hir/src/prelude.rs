use sodigy_name_analysis::NameKind;

// TODO: I want to call `intern_string(b"Int")`, but it's not a const function.
//       I can solve it by importing lazy_static, but I don't want external dependencies.
pub const PRELUDES: [(&'static [u8], NameKind); 6] = [
    // I'm just treating built-in types as struct. Maybe there's a better way.
    (b"Int", NameKind::Struct),
    (b"Bool", NameKind::Struct),
    (b"String", NameKind::Struct),
    (b"Char", NameKind::Struct),
    (b"True", NameKind::Use),  // `use Bool.True;`
    (b"False", NameKind::Use),  // `use Bool.True;`
];
