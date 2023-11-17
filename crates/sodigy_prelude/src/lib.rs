#![deny(unused_imports)]

use lazy_static::lazy_static;
use sodigy_intern::{InternedString, InternSession};
use sodigy_uid::Uid;
use std::collections::HashMap;

// it generates helper functions for preludes
// for ex, INT: (InternedString, Uid) has InternedString and Uid of `Int`
macro_rules! prelude_ref {
    ($name: ident, $sym: literal) => {
        lazy_static! {
            pub static ref $name: (InternedString, Uid) = {
                let mut intern_session = InternSession::new();
                let interned_string = intern_session.intern_string($sym.as_bytes().to_vec());

                (
                    interned_string,
                    *PRELUDES.get(&interned_string).unwrap(),
                )
            };
        }
    }
}

prelude_ref!(INT, "Int");
prelude_ref!(TYPE, "Type");
prelude_ref!(CHAR, "Char");
prelude_ref!(LIST, "List");
prelude_ref!(STRING, "String");
prelude_ref!(BOOL, "Bool");
prelude_ref!(FUNC, "Func");
prelude_ref!(OPTION, "Option");

lazy_static! {
    pub static ref PRELUDES: HashMap<InternedString, Uid> = {
        let mut intern_session = InternSession::new();
        let preludes = vec![
            ("Int", Uid::new_def().mark_prelude()),
            ("Type", Uid::new_def().mark_prelude()),
            ("Char", Uid::new_def().mark_prelude()),
            ("List", Uid::new_def().mark_prelude()),
            ("String", Uid::new_def().mark_prelude()),
            ("Bool", Uid::new_enum().mark_prelude()),
            ("Func", Uid::new_def().mark_prelude()),
            ("Option", Uid::new_enum().mark_prelude()),
        ];
        let mut result = HashMap::with_capacity(preludes.len());

        for (name, uid) in preludes.into_iter() {
            result.insert(
                intern_session.intern_string(name.into()),
                uid,
            );
        }

        result
    };
}
