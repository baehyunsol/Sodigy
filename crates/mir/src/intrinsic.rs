use sodigy_endec::{DecodeError, Endec};
use sodigy_error::FuncEffect;

// VIBE NOTE: Sonnet-4.5-thinking (via perplexity) wrote this code.
macro_rules! intrinsics {
    ($(($variant:ident, $lang_item:expr, $index:literal, $num_params:literal, $effect:ident)),* $(,)?) => {
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

            pub fn num_params(&self) -> usize {
                match self {
                    $(Intrinsic::$variant => $num_params,)*
                }
            }

            pub fn effect(&self) -> FuncEffect {
                match self {
                    $(Intrinsic::$variant => FuncEffect::$effect,)*
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

// There's a test named `verify_built_ins` in sodigy_driver which checks if this definition
// and the definition in the sodigy std match.
//
// You can find the documents in the sodigy std source code (search by their lang items!).
// In most cases, the built-in functions do not do any safety checks. For example,
// `DivInt` doesn't care about zero-divisions, and `IndexList` doesn't care about out-of-bounds.
// The compiler (or std) has to generate Sodigy code that does the safety checks.
intrinsics!(
    (NegInt          , "built_in.neg_int"           , 0    , 1   , Fn      ),
    (AddInt          , "built_in.add_int"           , 1    , 2   , Fn      ),
    (SubInt          , "built_in.sub_int"           , 2    , 2   , Fn      ),
    (MulInt          , "built_in.mul_int"           , 3    , 2   , Fn      ),
    (DivInt          , "built_in.div_int"           , 4    , 2   , Fn      ),
    (RemInt          , "built_in.rem_int"           , 5    , 2   , Fn      ),
    (LtInt           , "built_in.lt_int"            , 6    , 2   , Fn      ),
    (EqInt           , "built_in.eq_int"            , 7    , 2   , Fn      ),
    (GtInt           , "built_in.gt_int"            , 8    , 2   , Fn      ),
    (BitAndInt       , "built_in.bit_and_int"       , 9    , 2   , Fn      ),
    (BitOrInt        , "built_in.bit_or_int"        , 10   , 2   , Fn      ),
    (ShrInt          , "built_in.shr_int"           , 11   , 2   , Fn      ),
    (ShlInt          , "built_in.shl_int"           , 12   , 2   , Fn      ),
    (Ilog2Int        , "built_in.ilog2_int"         , 13   , 1   , Fn      ),
    (LtScalar        , "built_in.lt_scalar"         , 14   , 2   , Fn      ),
    (EqScalar        , "built_in.eq_scalar"         , 15   , 2   , Fn      ),
    (GtScalar        , "built_in.gt_scalar"         , 16   , 2   , Fn      ),
    (BitAndScalar    , "built_in.bit_and_scalar"    , 17   , 2   , Fn      ),
    (BitOrScalar     , "built_in.bit_or_scalar"     , 18   , 2   , Fn      ),
    (ScalarToInt     , "built_in.scalar_to_int"     , 19   , 1   , Fn      ),
    (IntToScalar     , "built_in.int_to_scalar"     , 20   , 1   , Fn      ),
    (IndexList       , "built_in.index_list"        , 21   , 2   , Fn      ),
    (LenList         , "built_in.len_list"          , 22   , 1   , Fn      ),
    (SliceList       , "built_in.slice_list"        , 23   , 3   , Fn      ),
    (SliceRightList  , "built_in.slice_right_list"  , 24   , 2   , Fn      ),
    (AppendList      , "built_in.append_list"       , 25   , 2   , Fn      ),
    (PrependList     , "built_in.prepend_list"      , 26   , 2   , Fn      ),
    (Exit            , "built_in.exit"              , 27   , 0   , Proc    ),
    (Panic           , "built_in.panic"             , 28   , 0   , Fn      ),

    // These are supposed to be `NdetProc`, but in order to implement some debug
    // functions, they're `Fn`.
    (Print           , "built_in.print"             , 29   , 1   , Fn      ),
    (EPrint          , "built_in.eprint"            , 30   , 1   , Fn      ),

    (RandomInt       , "built_in.random_int"        , 31   , 0   , NdetFn  ),
    (Sleep           , "built_in.sleep"             , 32   , 1   , Proc    ),
    (Nop             , "built_in.nop"               , 33   , 1   , Fn      ),
//   ^^^               ^^^^^^^^^^^^^^                 ^^     ^     ^^
//   |                 |                              |      |     |
//  (0)               (1)                            (2)    (3)   (4)
//
// (0): Enum variants (Rust)
// (1): lang items (Sodigy)
// (2): numeric index (endec)
// (3): number of parameters
// (4): effect
);
