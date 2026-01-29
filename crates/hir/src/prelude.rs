use crate::{Path, Use, Visibility};
use sodigy_name_analysis::{
    IdentWithOrigin,
    NameKind,
    NameOrigin,
    Namespace,
    UseCount,
};
use sodigy_parse::Field;
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};

// TODO: read `std/prelude.sdg` and actually import names from the file.
// TODO: I want to call `intern_string(b"Int")`, but it's not a const function.
//       I can solve it by importing lazy_static, but I don't want external dependencies.
pub const PRELUDES: [&'static [u8]; 13] = [
    b"Int",
    b"Number",
    b"Bool",
    b"String",
    b"Bytes",
    b"List",
    b"Char",
    b"Byte",
    b"True",   // `use Bool.True;`
    b"False",  // `use Bool.True;`

    // for type annotations
    b"Fn",
    b"PureFn",
    b"ImpureFn",
];

pub(crate) fn prelude_namespace(intermediate_dir: &str) -> Namespace {
    Namespace::Block {
        names: PRELUDES.iter().map(
            |name| (
                intern_string(name, intermediate_dir).unwrap(),
                (
                    Span::Prelude(intern_string(name, intermediate_dir).unwrap()),

                    // prelude `Int` is an implicit `use std.prelude.Int;`
                    NameKind::Use,

                    UseCount::new(),
                ),
            )
        ).collect(),
    }
}

pub fn use_prelude(name: InternedString) -> Use {
    // These are short strings, hence never fail.
    let prelude = intern_string(b"prelude", "").unwrap();
    let std = intern_string(b"std", "").unwrap();

    Use {
        visibility: Visibility::private(),
        keyword_span: Span::None,
        name,
        name_span: Span::Prelude(name),
        path: Path {
            id: IdentWithOrigin {
                id: std,
                span: Span::None,
                origin: NameOrigin::External,
                def_span: Span::Std,
            },
            fields: vec![
                Field::Name {
                    name: prelude,
                    name_span: Span::None,
                    dot_span: Span::None,
                    is_from_alias: false,
                },
                Field::Name {
                    name,
                    name_span: Span::None,
                    dot_span: Span::None,
                    is_from_alias: false,
                },
            ],
            types: vec![None; 3],
        },
    }
}
