// VIBE NOTE: Sonnet-4.5-thinking (via perplexity) wrote this code.
macro_rules! intrinsics {
    ($(($variant:ident, $lang_item:expr)),* $(,)?) => {
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
    };
}

// You can find the documents in the sodigy std source code (search by their lang items!).
intrinsics!(
    (AddInt, "built_in.add_int"),
    (SubInt, "built_in.sub_int"),
    (MulInt, "built_in.mul_int"),
    (DivInt, "built_in.div_int"),
    (RemInt, "built_in.rem_int"),
    (LtInt, "built_in.lt_int"),
    (EqInt, "built_in.eq_int"),
    (GtInt, "built_in.gt_int"),
    (Exit, "built_in.exit"),
    (Panic, "built_in.panic"),
    (Print, "built_in.print"),
    (EPrint, "built_in.eprint"),
);
