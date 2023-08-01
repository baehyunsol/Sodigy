use super::{Keyword, OpToken, Token, TokenKind};

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

macro_rules! token_op_checker {
    ($method: ident, $tok: ident) => {
        impl Token {
            pub fn $method(&self) -> bool {
                if let TokenKind::Operator(OpToken::$tok) = &self.kind {
                    true
                } else {
                    false
                }
            }
        }
    }
}

token_kind_impl!(Op, dot, Dot);
token_kind_impl!(Op, comma, Comma);
token_kind_impl!(Op, semi_colon, SemiColon);
token_kind_impl!(Op, colon, Colon);
token_kind_impl!(Op, assign, Assign);
token_kind_impl!(Op, right_arrow, RArrow);
token_kind_impl!(Op, opening_curly_brace, OpeningCurlyBrace);
token_kind_impl!(Op, closing_curly_brace, ClosingCurlyBrace);
token_kind_impl!(Key, keyword_if, If);
token_kind_impl!(Key, keyword_else, Else);
token_kind_impl!(Key, keyword_match, Match);
token_kind_impl!(Key, keyword_def, Def);
token_kind_impl!(Key, keyword_use, Use);
token_kind_impl!(Key, keyword_as, As);

token_op_checker!(is_dotdot, DotDot);
token_op_checker!(is_inclusive_range, InclusiveRange);
