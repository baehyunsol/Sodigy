use crate::module::ModulePath;
use crate::session::InternedString;
use crate::span::Span;
use crate::token::{Keyword, OpToken, Token, TokenKind};

#[derive(Clone)]
pub struct Use {
    path: ModulePath,
    alias: InternedString,
    span: Span,
}

pub fn use_case_to_tokens(Use { path, alias, span }: Use) -> Vec<Token> {
    // `use`, PATH, `as`, ALIAS, `;`
    let mut tokens = Vec::with_capacity(path.len() * 2 + 3);

    tokens.push(Token {
        span,
        kind: TokenKind::Keyword(Keyword::Use)
    });

    for token in path.tokens(span) {
        tokens.push(token);
    }

    tokens.push(Token {
        span,
        kind: TokenKind::Keyword(Keyword::As)
    });

    tokens.push(Token {
        span,
        kind: TokenKind::Identifier(alias)
    });

    tokens.push(Token {
        span,
        kind: TokenKind::Operator(OpToken::SemiColon)
    });

    tokens
}