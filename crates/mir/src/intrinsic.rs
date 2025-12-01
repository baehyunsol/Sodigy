use sodigy_endec::{DecodeError, Endec};

// VIBE NOTE: Sonnet-4.5-thinking (via perplexity) wrote this code.
macro_rules! intrinsics {
    ($(($variant:ident, $lang_item:expr, $index: literal)),* $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum Intrinsic {
            $($variant,)*
        }

        impl Intrinsic {
            pub const ALL: &'static [Intrinsic] = &[
                $(Intrinsic::$variant,)*
            ];

            pub const ALL_WITH_LANG_ITEM: &'static [(Intrinsic, &'static str)] = &[
                $((Intrinsic::$variant, $lang_item),)*
            ];

            pub fn lang_item(&self) -> &'static str {
                match self {
                    $(Intrinsic::$variant => $lang_item,)*
                }
            }

            pub fn from_lang_item(lang_item: &str) -> Option<Intrinsic> {
                match lang_item {
                    $($lang_item => Some(Intrinsic::$variant),)*
                    _ => None,
                }
            }
        }

        impl Endec for Intrinsic {
            fn encode_impl(&self, buffer: &mut Vec<u8>) {
                match self {
                    $(Intrinsic::$variant => { buffer.push($index); },)*
                }
            }

            fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
                match buffer.get(cursor) {
                    $(Some($index) => Ok((Intrinsic::$variant, cursor + 1)),)*
                    Some(n) => Err(DecodeError::InvalidEnumVariant(*n)),
                    None => Err(DecodeError::UnexpectedEof),
                }
            }
        }
    };
}

// You can find the documents in the sodigy std source code (search by their lang items!).
intrinsics!(
    (AddInt , "built_in.add_int" , 0),
    (SubInt , "built_in.sub_int" , 1),
    (MulInt , "built_in.mul_int" , 2),
    (DivInt , "built_in.div_int" , 3),
    (RemInt , "built_in.rem_int" , 4),
    (LtInt  , "built_in.lt_int"  , 5),
    (EqInt  , "built_in.eq_int"  , 6),
    (GtInt  , "built_in.gt_int"  , 7),
    (Exit   , "built_in.exit"    , 8),
    (Panic  , "built_in.panic"   , 9),
    (Print  , "built_in.print"   , 10),
    (EPrint , "built_in.eprint"  , 11),
//   ^^^^^^   ^^^^^^^^^^^^^^^^^    ^^
//   |        |                    |
//  (0)      (1)                  (2)
//
// (0): Enum variants (Rust)
// (1): lang items (Sodigy)
// (2): numeric index (endec)
);
