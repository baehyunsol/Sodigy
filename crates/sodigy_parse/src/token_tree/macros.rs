use super::TokenTreeKind;
use crate::punct::Punct;

macro_rules! punct_token {
    ($method: ident, $variant: ident) => {
        impl TokenTreeKind {
            pub fn $method() -> Self {
                TokenTreeKind::Punct(Punct::$variant)
            }
        }
    };
}

punct_token!(comma, Comma);
punct_token!(dot, Dot);
punct_token!(semi_colon, SemiColon);
punct_token!(colon, Colon);
punct_token!(gt, Gt);
punct_token!(lt, Lt);
punct_token!(assign, Assign);
punct_token!(r_arrow, RArrow);
punct_token!(sub, Sub);
