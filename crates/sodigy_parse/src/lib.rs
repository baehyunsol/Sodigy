#![deny(unused_imports)]

use log::info;
use sodigy_error::SodigyError;
use sodigy_lex::{CommentKind, LexSession, QuoteKind, Token, TokenKind};
use sodigy_number::SodigyNumber;
use sodigy_session::SodigySession;

mod delim;
mod error;
mod formatted_str;
mod punct;
mod session;
mod token_tree;
mod warn;

#[cfg(test)]
mod tests;

pub use delim::{Delim, DelimStart};
pub use error::ParseError;
use formatted_str::parse_str;
pub use formatted_str::FormattedStringElement;
pub use punct::Punct;
pub use session::ParseSession;
pub use token_tree::{TokenTree, TokenTreeKind};
pub use warn::ParseWarning;

pub fn from_tokens(tokens: &[Token], session: &mut ParseSession, lex_session: &mut LexSession) -> Result<(), ()> {
    info!(
        "sodigy_parse::from_tokens(), first few tokens are: {:?}",
        &tokens[..4.min(tokens.len())],
    );

    let mut index = 0;
    let mut group_stack = vec![];
    let mut has_macro = false;

    loop {
        match tokens.get(index) {
            Some(token) => match &token.kind {
                TokenKind::Comment { kind: CommentKind::Doc, content } => {
                    let content = session.intern_string(content.as_bytes().to_vec());

                    session.push_result(TokenTree {
                        kind: TokenTreeKind::DocComment(content),
                        span: token.span,
                    });
                },
                TokenKind::Comment { .. } => {
                    /* nop */
                },
                TokenKind::String { kind, content, is_fstring } /* prefixed string literals are already handled */ => {
                    if *is_fstring {
                        let mut error = ParseError::fstring_without_prefix(false, token.span);
                        error.set_message(
                            "Also, use double quotes for a format-string. Single quotes are for a character literals.".to_string()
                        );

                        session.push_error(error);

                        // don't return here! find more errors!
                        // return Err(());
                    }

                    let content = session.intern_string(content.as_bytes().to_vec());

                    session.push_result(TokenTree {
                        kind: TokenTreeKind::String {
                            kind: *kind,
                            content,
                            is_binary: false,
                        },
                        span: token.span,
                    });
                },
                TokenKind::Whitespace => {
                    /* nop */
                },
                TokenKind::Punct(p1) => {
                    if *p1 == b'`' {
                        // field modifier
                        match tokens.get(index + 1) {
                            Some(Token { kind: TokenKind::Identifier(id), span: span2 }) => {
                                session.push_result(TokenTree {
                                    kind: TokenTreeKind::Punct(Punct::FieldModifier(*id)),
                                    span: token.span.merge(*span2),
                                });

                                index += 1;
                            },
                            _ => {
                                session.push_error(ParseError::lonely_backtick(token.span));
                                return Err(());
                            },
                        }
                    }

                    else if *p1 == b'\\' {
                        match tokens.get(index + 1) {
                            Some(Token { kind: TokenKind::Grouper(g), .. }) => {
                                group_stack.push(DelimStart::new_prefix(*g, session.get_results().len(), token.span, b'\\'));
                                index += 1;
                            },
                            _ => {
                                session.push_error(ParseError::lonely_backslash(token.span));
                                return Err(());
                            },
                        }
                    }

                    else {
                        match tokens.get(index + 1) {
                            Some(Token { kind: TokenKind::Punct(p2), span: span2 }) => match Punct::try_from_two_chars(*p1, *p2) {
                                Some(p) => {
                                    let span = token.span.merge(*span2);

                                    match p {
                                        Punct::DotDot => match tokens.get(index + 2) {
                                            // for now, this is the only 3-chars punct
                                            Some(Token { kind: TokenKind::Punct(b'~'), span: span3 }) => {
                                                let span = span.merge(*span3);
                                                session.push_result(TokenTree {
                                                    kind: TokenTreeKind::Punct(Punct::InclusiveRange),
                                                    span,
                                                });

                                                index += 2;
                                            },
                                            Some(Token { kind: TokenKind::Punct(b'.'), span: span3 }) => {
                                                let span = span.merge(*span3);
                                                session.push_error(ParseError::three_dots(span));
                                                return Err(());
                                            },
                                            _ => {
                                                session.push_result(TokenTree {
                                                    kind: TokenTreeKind::Punct(p),
                                                    span,
                                                });

                                                index += 1;
                                            },
                                        },
                                        _ => {
                                            session.push_result(TokenTree {
                                                kind: TokenTreeKind::Punct(p),
                                                span,
                                            });

                                            index += 1;
                                        },
                                    }
                                },
                                _ => {
                                    session.push_result(TokenTree {
                                        kind: TokenTreeKind::Punct((*p1).try_into().unwrap()),  // lexer assures that it doesn't fail
                                        span: token.span,
                                    });
                                },
                            },
                            _ => {
                                if is_macro(tokens, index) {
                                    has_macro = true;
                                }

                                session.push_result(TokenTree {
                                    kind: TokenTreeKind::Punct((*p1).try_into().unwrap()),  // lexer assures that it doesn't fail
                                    span: token.span,
                                });
                            },
                        }
                    }
                },
                TokenKind::Grouper(g) => match g {
                    b'{' | b'[' | b'('  => {
                        group_stack.push(DelimStart::new(*g, session.get_results().len(), token.span));
                    },
                    b'}' | b']' | b')'  => {
                        match group_stack.pop() {
                            Some(ds) => if ds.kind == Delim::from(*g) {
                                let span = ds.span.merge(token.span);
                                let mut tokens = Vec::with_capacity(session.get_results().len() - ds.index);

                                // TODO: there must be a better/neater/prettier function
                                while session.get_results().len() > ds.index {
                                    tokens.push(session.pop_result().unwrap());
                                }

                                session.push_result(TokenTree {
                                    kind: TokenTreeKind::Group {
                                        tokens: tokens.into_iter().rev().collect(),
                                        delim: ds.kind,
                                        prefix: ds.prefix,
                                    },
                                    span,
                                });
                            } else {
                                session.push_error(ParseError::unfinished_delim(ds.start_char(), ds.span));
                                return Err(());
                            },
                            None => {
                                session.push_error(ParseError::mismatch_delim(*g, token.span));
                                return Err(());
                            },
                        }
                    },
                    _ => unreachable!(),
                },
                TokenKind::Number(lit) => {
                    let interned_numeric = session.intern_numeric(SodigyNumber::from_string(lit));

                    session.push_result(TokenTree {
                        kind: TokenTreeKind::Number(interned_numeric),
                        span: token.span,
                    });
                },
                TokenKind::Identifier(id) => {
                    let short_string = id.try_unwrap_short_string();
                    let (is_b, is_f) = if let Some((1, bytes)) = short_string {
                        (
                            &bytes[0..1] == b"b",
                            &bytes[0..1] == b"f",
                        )
                    } else {
                        (false, false)
                    };

                    if is_b || is_f {

                        // `b"123"` is okay, but `b "123"` is not.
                        match tokens.get(index + 1) {
                            Some(Token { kind: TokenKind::String { kind: quote_kind, content, is_fstring }, span: span2 }) => {
                                let span2 = *span2;
                                let quote_kind = *quote_kind;

                                if quote_kind == QuoteKind::Double {
                                    if is_b {
                                        if *is_fstring {
                                            session.push_error(ParseError::fstring_without_prefix(true, token.span));
                                            return Err(());
                                        }

                                        let content = session.intern_string(content.as_bytes().to_vec());

                                        session.push_result(
                                            TokenTree {
                                                kind: TokenTreeKind::String {
                                                    kind: QuoteKind::Double,
                                                    content,
                                                    is_binary: true,
                                                },
                                                span: token.span.merge(span2),
                                            }
                                        );
                                    }

                                    else {
                                        let f_s = parse_str(
                                            content.as_bytes(),
                                            span2.start().offset(1),  // skip `"`
                                            lex_session,
                                            session,
                                        )?;

                                        session.push_result(TokenTree {
                                            kind: TokenTreeKind::FormattedString(f_s),
                                            span: token.span.merge(span2),
                                        });
                                    }

                                    index += 1;
                                }

                                else {
                                    session.push_error(ParseError::fstring_single_quote(span2));
                                    return Err(());
                                }
                            },
                            _ => {
                                let token = match id.try_into_keyword() {
                                    Some(k) => TokenTree {
                                        kind: TokenTreeKind::Keyword(k),
                                        span: token.span,
                                    },
                                    None => TokenTree {
                                        kind: TokenTreeKind::Identifier(*id),
                                        span: token.span,
                                    },
                                };

                                session.push_result(token);
                            },
                        }
                    }

                    else {
                        match id.try_into_keyword() {
                            Some(k) => {
                                session.push_result(TokenTree {
                                    kind: TokenTreeKind::Keyword(k),
                                    span: token.span,
                                });
                            },
                            None => {
                                session.push_result(TokenTree {
                                    kind: TokenTreeKind::Identifier(*id),
                                    span: token.span,
                                });
                            },
                        }
                    }
                },
            },
            None => match group_stack.pop() {
                Some(ds) => {
                    session.push_error(ParseError::unfinished_delim(ds.start_char(), ds.span));

                    return Err(());
                },
                _ => {
                    session.err_if_has_err()?;

                    if has_macro {
                        session.replace_macro_tokens()?;
                    }

                    return Ok(());
                }
            },
        }

        index += 1;
    }
}

// it only checks whether the current tokens are `@[`.
// if the current tokens are `@[` but not a valid macro, `session.try_expand_macro` would throw an error.
fn is_macro(tokens: &[Token], mut index: usize) -> bool {
    if let Some(
        Token {
            kind: TokenKind::Punct(b'@'),
            ..
        }
    ) = tokens.get(index) {
        index += 1;

        while let Some(Token {
            kind: TokenKind::Whitespace,
            ..
        }) = tokens.get(index) {
            index += 1;
        }

        if let Some(Token {
            kind: TokenKind::Grouper(b'['),
            ..
        }) = tokens.get(index) {
            true
        }

        else {
            false
        }
    }

    else {
        false
    }
}
