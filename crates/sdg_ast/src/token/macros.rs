use super::{Keyword, OpToken, TokenKind};

macro_rules! token_kind_impl {
    (Op, $method: ident, $var: ident) => {
        impl TokenKind {
        
            pub fn $method() -> Self {
                TokenKind::Operator(OpToken::$var)
            }
        
        }
    };
    (Key, $method: ident, $var: ident) => {
        impl TokenKind {
        
            pub fn $method() -> Self {
                TokenKind::Keyword(Keyword::$var)
            }
        
        }
    };
}

token_kind_impl!(Op, dot, Dot);
token_kind_impl!(Op, comma, Comma);
token_kind_impl!(Op, semi_colon, SemiColon);
token_kind_impl!(Op, assign, Assign);
token_kind_impl!(Key, keyword_if, If);
token_kind_impl!(Key, keyword_else, Else);
token_kind_impl!(Key, keyword_def, Def);
token_kind_impl!(Key, keyword_use, Use);
token_kind_impl!(Key, keyword_as, As);