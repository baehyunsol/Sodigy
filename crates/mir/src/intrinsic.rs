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
    (NegInt , "built_in.neg_int" , 0),
    (AddInt , "built_in.add_int" , 1),
    (SubInt , "built_in.sub_int" , 2),
    (MulInt , "built_in.mul_int" , 3),
    (DivInt , "built_in.div_int" , 4),
    (RemInt , "built_in.rem_int" , 5),
    (LtInt  , "built_in.lt_int"  , 6),
    (EqInt  , "built_in.eq_int"  , 7),
    (GtInt  , "built_in.gt_int"  , 8),
    (Exit   , "built_in.exit"    , 9),
    (Panic  , "built_in.panic"   , 10),
    (Print  , "built_in.print"   , 11),
    (EPrint , "built_in.eprint"  , 12),
//   ^^^^^^   ^^^^^^^^^^^^^^^^^    ^^
//   |        |                    |
//  (0)      (1)                  (2)
//
// (0): Enum variants (Rust)
// (1): lang items (Sodigy)
// (2): numeric index (endec)
);
