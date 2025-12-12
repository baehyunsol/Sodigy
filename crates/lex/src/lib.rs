// NOTE: The lexer loads the entire file to memory. There's no input buffer.
//       You know, ... it's 21st century! Everything's gonna be fine.

use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_file::File;
use sodigy_number::{Base, InternedNumber, InternedNumberValue, intern_number};
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};
use sodigy_token::{Delim, Keyword, Punct, Token, TokenKind, TokensOrString};
use std::num::IntErrorKind;

mod endec;
mod session;

pub use session::Session;

#[derive(Debug)]
pub(crate) enum LexState {
    Init,

    // `StringPrefix` first parses prefix `b`, `f`, `r` or `re` before the literal
    // then `StringInit` counts the number of double quote characters
    // then `String` parses the content of the literal
    StringPrefix,
    StringInit {
        binary: bool,
        format: bool,
        raw: bool,

        // If regex is set, it implies that
        // binary is false
        // format is false
        // raw is true
        regex: bool,
    },
    String {
        format: bool,
        binary: bool,
        raw: bool,
        quote_count: usize,

        // It doesn't affect the parsing. It only affects the result.
        regex: bool,
    },
    Char {
        binary: bool,
    },

    Ident,
    FieldModifier,
    Integer(Base),
    Fraction,
    Exp,
    LineComment,
    DocComment {
        top_level: bool,
    },
    BlockComment,
}

pub fn lex(
    file: File,
    input: Vec<u8>,
    intermediate_dir: String,
    is_std: bool,
) -> Session {
    let mut session = Session {
        file,
        input_bytes: input,
        state: LexState::Init,
        cursor: 0,
        tokens: vec![],
        intermediate_dir,
        is_std,
        group_stack: vec![],
        token_start: 0,
        buffer1: vec![],
        buffer2: vec![],
        fstring_buffer: vec![],
        errors: vec![],
        warnings: vec![],
    };

    loop {
        match session.step() {
            Ok(true) => { break; },
            Ok(false) => {},
            Err(e) => {
                session.errors.push(e);
                break;
            },
        }
    }

    if session.errors.is_empty() {
        session.group_tokens();
    }

    session
}

impl Session {
    fn step(&mut self) -> Result<bool, Error> {  // returns Ok(true) if it reaches Eof
        match self.state {
            LexState::Init => match (self.input_bytes.get(self.cursor), self.input_bytes.get(self.cursor + 1), self.input_bytes.get(self.cursor + 2)) {
                (Some(b'a'..=b'z'), Some(b'a'..=b'z'), Some(b'"' | b'\'')) |
                (Some(b'a'..=b'z'), Some(b'"' | b'\''), _) |
                (Some(b'"' | b'\''), _, _) => {
                    self.buffer1.clear();
                    self.fstring_buffer.clear();
                    self.token_start = self.cursor;
                    self.state = LexState::StringPrefix;
                },
                (Some(x @ (b'a'..=b'z' | b'A'..=b'Z' | b'_')), _, _) => {
                    self.buffer1.clear();
                    self.buffer1.push(*x);

                    self.token_start = self.cursor;
                    self.state = LexState::Ident;
                    self.cursor += 1;
                },
                (Some(b'`'), Some(y @ (b'a'..=b'z' | b'A'..=b'Z' | b'_')), _) => {
                    self.buffer1.clear();
                    self.buffer1.push(*y);

                    self.token_start = self.cursor;
                    self.state = LexState::FieldModifier;
                    self.cursor += 2;
                },
                // It's `Number + Punct("..")`, and we have to prevent the lexer reading it `DottedNumber + Punct(".")`.
                (Some(x @ b'0'..=b'9'), Some(b'.'), Some(b'.')) => {
                    self.tokens.push(Token {
                        kind: TokenKind::Number(InternedNumber::from_u32((*x - b'0') as u32, true /* is_integer */)),
                        span: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 1,
                        ),
                    });
                    self.cursor += 1;
                },
                (Some(b'0'), Some(y @ (b'x' | b'X' | b'o' | b'O' | b'b' | b'B')), _) => {
                    let base = match y {
                        b'x' | b'X' => Base::Hexadecimal,
                        b'o' | b'O' => Base::Octal,
                        b'b' | b'B' => Base::Binary,
                        _ => unreachable!(),
                    };

                    self.buffer1.clear();
                    self.token_start = self.cursor;
                    self.state = LexState::Integer(base);
                    self.cursor += 2;
                },
                (Some(b'0'..=b'9'), Some(y @ (b'a'..=b'z' | b'A'..=b'Z')), _) => {
                    return Err(Error {
                        kind: ErrorKind::InvalidNumberLiteral,
                        spans: Span::range(
                            self.file,
                            self.cursor + 1,
                            self.cursor + 2,
                        ).simple_error(),
                        note: Some(format!("`{}` is not a valid prefix. Valid ones are `x`, `X`, `o`, `O`, `b` and `B`.", *y as char)),
                    });
                },
                (Some(b'0'), Some(b'.'), _) => {
                    self.buffer1.clear();
                    self.buffer1.push(b'0');
                    self.buffer2.clear();
                    self.token_start = self.cursor;
                    self.state = LexState::Fraction;
                    self.cursor += 2;
                },
                (Some(b'0'), Some(b'0'..=b'9'), _) => {
                    return Err(Error {
                        kind: ErrorKind::InvalidNumberLiteral,
                        spans: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 1,
                        ).simple_error(),
                        note: Some(String::from("Leading zeros in decimal literals are not permitted.")),
                    });
                },
                (Some(b'0'), _, _) => {
                    self.tokens.push(Token {
                        kind: TokenKind::Number(InternedNumber::from_u32(0, true /* is_integer */)),
                        span: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 1,
                        ),
                    });
                    self.cursor += 1;
                },
                (Some(x @ (b'1'..=b'9')), _, _) => {
                    self.buffer1.clear();
                    self.buffer1.push(*x);

                    self.token_start = self.cursor;
                    self.state = LexState::Integer(Base::Decimal);
                    self.cursor += 1;
                },
                (Some(b'#'), Some(b'['), _) => {
                    let opening_span = Span::range(
                        self.file,
                        self.cursor,
                        self.cursor + 2,
                    );
                    self.group_stack.push((b']', opening_span));
                    self.tokens.push(Token {
                        kind: TokenKind::GroupDelim {
                            delim: Some(Delim::Decorator),
                            id: opening_span,
                        },
                        span: opening_span,
                    });
                    self.cursor += 2;
                },
                (Some(b'#'), Some(b'!'), Some(b'[')) => {
                    let opening_span = Span::range(
                        self.file,
                        self.cursor,
                        self.cursor + 3,
                    );
                    self.group_stack.push((b']', opening_span));
                    self.tokens.push(Token {
                        kind: TokenKind::GroupDelim {
                            delim: Some(Delim::ModuleDecorator),
                            id: opening_span,
                        },
                        span: opening_span,
                    });
                    self.cursor += 3;
                },
                (Some(b'#'), _, _) => {
                    let token_start = self.cursor;
                    self.cursor += 1;
                    let mut base = Base::Decimal;
                    let mut buffer = vec![];

                    if let Some(x @ (b'x' | b'X' | b'o' | b'O' | b'b' | b'B')) = self.input_bytes.get(self.cursor) {
                        base = match x {
                            b'x' | b'X' => Base::Hexadecimal,
                            b'o' | b'O' => Base::Octal,
                            b'b' | b'B' => Base::Binary,
                            _ => unreachable!(),
                        };

                        self.cursor += 1;
                    }

                    loop {
                        match self.input_bytes.get(self.cursor) {
                            // `b'g'..=b'z'` is always error, but it matches the
                            // range so that it can generate a better error message
                            Some(x @ (b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z')) => {
                                if !base.is_valid_digit(*x) {
                                    return Err(Error {
                                        kind: ErrorKind::InvalidByteLiteral,
                                        spans: Span::range(
                                            self.file,
                                            self.cursor,
                                            self.cursor + 1,
                                        ).simple_error(),
                                        note: Some(base.invalid_digit_error_message(*x)),
                                    });
                                }

                                buffer.push(*x);
                                self.cursor += 1;
                            },
                            Some(b'_') => {
                                self.cursor += 1;
                            },
                            _ => {
                                break;
                            },
                        }
                    }

                    if buffer.is_empty() {
                        return Err(Error {
                            kind: ErrorKind::InvalidByteLiteral,
                            spans: Span::range(
                                self.file,
                                token_start,
                                self.cursor,
                            ).simple_error(),
                            note: None,
                        });
                    }

                    let n = intern_number(base, &buffer, &[], 0, true /* is_integer */);

                    match n.value {
                        InternedNumberValue::SmallInt(n @ 0..=255) => {
                            self.tokens.push(Token {
                                kind: TokenKind::Byte(n as u8),
                                span: Span::range(
                                    self.file,
                                    token_start,
                                    self.cursor,
                                ),
                            });
                        },
                        _ => {
                            return Err(Error {
                                kind: ErrorKind::InvalidByteLiteral,
                                spans: Span::range(
                                    self.file,
                                    token_start,
                                    self.cursor,
                                ).simple_error(),
                                note: Some(String::from("A byte must be in range #0..=#255.")),
                            });
                        },
                    }
                },
                (Some(b'/'), Some(b'/'), Some(z @ (b'/' | b'!'))) => {
                    let top_level = *z == b'!';
                    self.token_start = self.cursor;
                    self.state = LexState::DocComment { top_level: top_level };
                    self.cursor += 3;
                },
                (Some(b'/'), Some(b'/'), _) => {
                    self.state = LexState::LineComment;
                    self.cursor += 2;
                },
                (Some(b'/'), Some(b'*'), _) => {
                    self.token_start = self.cursor;
                    self.state = LexState::BlockComment;
                    self.cursor += 2;
                },
                (Some(b' ' | b'\t' | b'\n'), _, _) => {
                    self.cursor += 1;
                },
                (Some(x @ (b'[' | b'{' | b'(')), _, _) => {
                    let (opening_delim, closing_delim) = match x {
                        b'[' => (Delim::Bracket, b']'),
                        b'{' => (Delim::Brace, b'}'),
                        b'(' => (Delim::Parenthesis, b')'),
                        _ => unreachable!(),
                    };
                    let opening_span = Span::range(
                        self.file,
                        self.cursor,
                        self.cursor + 1,
                    );
                    self.group_stack.push((closing_delim, opening_span));
                    self.tokens.push(Token {
                        kind: TokenKind::GroupDelim {
                            delim: Some(opening_delim),
                            id: opening_span,
                        },
                        span: opening_span,
                    });
                    self.cursor += 1;
                },
                (Some(b'\\'), Some(b'('), _) => {
                    let opening_span = Span::range(
                        self.file,
                        self.cursor,
                        self.cursor + 2,
                    );
                    self.group_stack.push((b')', opening_span));
                    self.tokens.push(Token {
                        kind: TokenKind::GroupDelim {
                            delim: Some(Delim::Lambda),
                            id: opening_span,
                        },
                        span: opening_span,
                    });
                    self.cursor += 2;
                },
                (Some(x @ (b']' | b'}' | b')')), _, _) => match self.group_stack.pop() {
                    Some((delim, span)) if delim == *x => {
                        self.tokens.push(Token {
                            kind: TokenKind::GroupDelim {
                                delim: None,
                                id: span,
                            },
                            span: Span::range(
                                self.file,
                                self.cursor,
                                self.cursor + 1,
                            ),
                        });
                        self.cursor += 1;
                    },
                    Some((delim, _)) => {
                        return Err(Error {
                            kind: ErrorKind::UnmatchedGroup {
                                expected: delim,
                                got: *x,
                            },
                            spans: Span::range(
                                self.file,
                                self.cursor,
                                self.cursor + 1,
                            ).simple_error(),
                            ..Error::default()
                        });
                    },
                    None => {
                        return Err(Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: ErrorToken::Any,
                                got: ErrorToken::Character(*x),
                            },
                            spans: Span::eof(self.file).simple_error(),
                            ..Error::default()
                        });
                    },
                },
                // This is the only 3-character punct in the current spec
                (Some(b'.'), Some(b'.'), Some(b'=')) => {
                    self.tokens.push(Token {
                        kind: TokenKind::Punct(Punct::DotDotEq),
                        span: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 3,
                        ),
                    });
                    self.cursor += 3;
                },
                (
                    Some(x @ (b'!' | b'&' | b'+' | b'-' | b'.' | b'<' | b'=' | b'>' | b'|')),
                    Some(y @ (b'&' | b'+' | b'.' | b'<' | b'=' | b'>' | b'|')),
                    _,
                ) => {
                    let punct = match (x, y) {
                        (b'!', b'=') => Some(Punct::Neq),
                        (b'&', b'&') => Some(Punct::AndAnd),
                        (b'+', b'+') => Some(Punct::Concat),
                        (b'+', b'>') => Some(Punct::Prepend),
                        (b'-', b'>') => Some(Punct::ReturnType),
                        (b'.', b'.') => Some(Punct::DotDot),
                        (b'<', b'+') => Some(Punct::Append),
                        (b'<', b'<') => Some(Punct::Shl),
                        (b'<', b'=') => Some(Punct::Leq),
                        (b'=', b'=') => Some(Punct::Eq),
                        (b'=', b'>') => Some(Punct::Arrow),
                        (b'>', b'=') => Some(Punct::Geq),
                        (b'>', b'>') => Some(Punct::Shr),
                        (b'|', b'|') => Some(Punct::OrOr),
                        _ => None,
                    };

                    match punct {
                        Some(p) => {
                            self.tokens.push(Token {
                                kind: TokenKind::Punct(p),
                                span: Span::range(
                                    self.file,
                                    self.cursor,
                                    self.cursor + 2,
                                ),
                            });
                            self.cursor += 2;
                        },
                        None => {
                            // It'd be 99.9% parse error, but lexer doesn't care about that.
                            self.tokens.push(Token {
                                kind: TokenKind::Punct((*x).into()),
                                span: Span::range(
                                    self.file,
                                    self.cursor,
                                    self.cursor + 1,
                                ),
                            });
                            self.tokens.push(Token {
                                kind: TokenKind::Punct((*y).into()),
                                span: Span::range(
                                    self.file,
                                    self.cursor + 1,
                                    self.cursor + 2,
                                ),
                            });
                            self.cursor += 2;
                        },
                    }
                },
                (Some(x @ (
                    b'!' | b'$' | b'%' | b'&' | b'*' |
                    b'+' | b',' | b'-' | b'.' | b'/' |
                    b':' | b';' | b'<' | b'=' | b'>' |
                    b'?' | b'@' | b'^' | b'|' | b'~'
                )), _, _) => {
                    self.tokens.push(Token {
                        kind: TokenKind::Punct((*x).into()),
                        span: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 1,
                        ),
                    });
                    self.cursor += 1;
                },
                // It's either a non-ascii identifier or an error.
                (Some(192..), _, _) => {
                    self.buffer1.clear();
                    self.token_start = self.cursor;
                    self.state = LexState::Ident;
                },
                (Some(x), _, _) => panic!("TODO: {:?}", *x as char),
                (None, _, _) => {
                    if let Some((delim, span)) = self.group_stack.pop() {
                        return Err(Error {
                            kind: ErrorKind::UnclosedDelimiter(delim),
                            spans: span.simple_error(),
                            ..Error::default()
                        });
                    }

                    else {
                        return Ok(true);
                    }
                },
            },
            // b"abc" -> binary string
            // b'a' -> binary char
            // f"abc" -> formatted string
            // r"abc" -> raw string
            // re"abc" -> regex
            // br"abc", rb"abc" -> binary raw string
            // fr"abc", rf"abc" -> formatted raw string
            LexState::StringPrefix => match (self.input_bytes.get(self.cursor), self.input_bytes.get(self.cursor + 1), self.input_bytes.get(self.cursor + 2)) {
                // Cannot use the same prefix multiple times.
                (Some(x @ (b'b' | b'f' | b'r')), Some(y @ (b'b' | b'f' | b'r')), Some(z @ (b'"' | b'\''))) if x == y => {
                    return Err(Error {
                        kind: if *z == b'"' {
                            ErrorKind::InvalidStringLiteralPrefix
                        } else {
                            ErrorKind::InvalidCharLiteralPrefix
                        },
                        spans: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 2,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
                (Some(b'b'), Some(b'f'), Some(z @ (b'"' | b'\''))) |
                (Some(b'f'), Some(b'b'), Some(z @ (b'"' | b'\''))) => {
                    return Err(Error {
                        kind: if *z == b'"' {
                            ErrorKind::InvalidStringLiteralPrefix
                        } else {
                            ErrorKind::InvalidCharLiteralPrefix
                        },
                        spans: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 2,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
                (Some(b'b'), Some(b'r'), Some(b'"')) |
                (Some(b'r'), Some(b'b'), Some(b'"')) => {
                    self.state = LexState::StringInit {
                        binary: true,
                        format: false,
                        raw: true,
                        regex: false,
                    };
                    self.cursor += 2;
                },
                // A binary char is okay, but a raw char is not.
                (Some(x @ b'b'), Some(b'r'), Some(b'\'')) |
                (Some(x @ b'r'), Some(b'b'), Some(b'\'')) => {
                    let error_span = if *x == b'b' {
                        Span::range(
                            self.file,
                            self.cursor + 1,
                            self.cursor + 2,
                        )
                    } else {
                        Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 1,
                        )
                    };
                    return Err(Error {
                        kind: ErrorKind::InvalidCharLiteralPrefix,
                        spans: error_span.simple_error(),
                        ..Error::default()
                    });
                },
                (Some(b'f'), Some(b'r'), Some(b'"')) |
                (Some(b'r'), Some(b'f'), Some(b'"')) => {
                    self.state = LexState::StringInit {
                        binary: false,
                        format: true,
                        raw: true,
                        regex: false,
                    };
                    self.cursor += 2;
                },
                // `f` and `r` are both invalid for a char literal
                (Some(b'f'), Some(b'r'), Some(b'\'')) |
                (Some(b'r'), Some(b'f'), Some(b'\'')) => {
                    return Err(Error {
                        kind: ErrorKind::InvalidCharLiteralPrefix,
                        spans: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 2,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
                (Some(b'r'), Some(b'e'), Some(b'"')) => {
                    self.state = LexState::StringInit {
                        binary: false,
                        format: false,
                        raw: true,
                        regex: true,
                    };
                    self.cursor += 2;
                },
                // `re` is invalid for a char literal
                (Some(b'r'), Some(b'e'), Some(b'\'')) => {
                    return Err(Error {
                        kind: ErrorKind::InvalidCharLiteralPrefix,
                        spans: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 2,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
                (Some(x @ (b'b' | b'f' | b'r')), Some(b'"'), _) => {
                    self.state = LexState::StringInit {
                        binary: *x == b'b',
                        format: *x == b'f',
                        raw: *x == b'r',
                        regex: false,
                    };
                    self.cursor += 1;
                },
                (Some(b'b'), Some(b'\''), _) => {
                    self.state = LexState::StringInit {
                        binary: true,
                        format: false,
                        raw: false,
                        regex: false,
                    };
                    self.cursor += 1;
                },
                (Some(b'f' | b'r'), Some(b'\''), _) => {
                    return Err(Error {
                        kind: ErrorKind::InvalidCharLiteralPrefix,
                        spans: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 1,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
                (Some(b'"' | b'\''), _, _) => {
                    self.state = LexState::StringInit {
                        binary: false,
                        format: false,
                        raw: false,
                        regex: false,
                    };
                },
                (Some(b'a'..=b'z'), Some(b'a'..=b'z'), Some(z @ (b'"' | b'\''))) => {
                    return Err(Error {
                        kind: if *z == b'"' {
                            ErrorKind::InvalidStringLiteralPrefix
                        } else {
                            ErrorKind::InvalidCharLiteralPrefix
                        },
                        spans: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 2,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
                (Some(b'a'..=b'z'), Some(y @ (b'"' | b'\'')), _) => {
                    return Err(Error {
                        kind: if *y == b'"' {
                            ErrorKind::InvalidStringLiteralPrefix
                        } else {
                            ErrorKind::InvalidCharLiteralPrefix
                        },
                        spans: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 1,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
                _ => unreachable!(),
            },
            // `LexState::StringInit` doesn't care even if a char literal has multiple characters.
            // `LexState::Char` will throw an error for that.
            LexState::StringInit { binary, format, raw, regex } => match (
                self.input_bytes.get(self.cursor),
                self.input_bytes.get(self.cursor + 1),
                self.input_bytes.get(self.cursor + 2),
            ) {
                (Some(b'"'), Some(b'"'), Some(b'"')) => {
                    let quote_count = count_quotes(&self.input_bytes, self.cursor).unwrap_or(256);

                    if quote_count % 2 == 0 && quote_count > 254 || quote_count % 2 == 1 && quote_count > 127 {
                        return Err(Error {
                            kind: ErrorKind::TooManyQuotes,
                            spans: Span::range(
                                self.file,
                                // I don't want to highlight all the quotes... it's *TooMany*Quotes
                                self.cursor,
                                self.cursor + 1,
                            ).simple_error(),
                            ..Error::default()
                        });
                    }

                    match quote_count {
                        // an empty string literal
                        // for example, if double-quote appears 6 times,
                        // the first 3 starts the literal and the last 3 ends the literal
                        x if x % 4 == 2 => {
                            let token_kind = if format {
                                // TokenKind::FormattedString {}
                                todo!()
                            } else {
                                TokenKind::String {
                                    binary,
                                    raw,
                                    regex,
                                    s: InternedString::empty(),
                                }
                            };

                            self.tokens.push(Token {
                                kind: token_kind,
                                span: Span::range(
                                    self.file,
                                    self.cursor,
                                    self.cursor + quote_count,
                                ),
                            });
                            self.state = LexState::Init;
                            self.cursor += quote_count;
                        },

                        // invalid
                        // a string literal must start with an odd number of quotes
                        x if x % 4 == 0 => {
                            return Err(Error {
                                kind: ErrorKind::WrongNumberOfQuotesInRawStringLiteral,
                                spans: Span::range(
                                    self.file,
                                    self.cursor,
                                    self.cursor + quote_count,
                                ).simple_error(),
                                ..Error::default()
                            });
                        },

                        // start of a literal
                        _ => {
                            self.state = LexState::String {
                                format,
                                binary,
                                raw,
                                quote_count,
                                regex,
                            };
                            self.cursor += quote_count;
                        },
                    }
                },
                // an empty string literal
                (Some(b'"'), Some(b'"'), _) => {
                    let token_kind = if format {
                        TokenKind::FormattedString {
                            raw,
                            elements: vec![],
                        }
                    } else {
                        TokenKind::String {
                            binary,
                            raw,
                            regex,
                            s: InternedString::empty(),
                        }
                    };
                    self.tokens.push(Token {
                        kind: token_kind,
                        span: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 2,
                        ),
                    });
                    self.state = LexState::Init;
                    self.cursor += 2;
                },
                // an empty char literal -> error!
                (Some(b'\''), Some(b'\''), _) => {
                    return Err(Error {
                        kind: ErrorKind::EmptyCharLiteral,
                        spans: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 2,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
                (Some(b'"'), _, _) => {
                    self.state = LexState::String {
                        format,
                        binary,
                        raw,
                        quote_count: 1,
                        regex,
                    };
                    self.cursor += 1;
                },
                (Some(b'\''), _, _) => {
                    self.state = LexState::Char { binary };
                    self.cursor += 1;
                },
                _ => unreachable!(),
            },
            LexState::String { format, binary, raw: true, quote_count, regex } => match (
                self.input_bytes.get(self.cursor),
                self.input_bytes.get(self.cursor + 1),
                self.input_bytes.get(self.cursor + 2),
            ) {
                (Some(b'{'), Some(b'{'), _) if format => {
                    self.buffer1.push(b'{');
                    self.cursor += 2;
                },
                (Some(b'}'), Some(b'}'), _) if format => {
                    self.buffer1.push(b'}');
                    self.cursor += 2;
                },
                (Some(b'{'), _, _) if format => {
                    let interned = intern_string(&self.buffer1, &self.intermediate_dir).unwrap();
                    self.fstring_buffer.push(TokensOrString::String(interned));
                    self.lex_formatted_string()?;
                    self.buffer1.clear();
                },
                (Some(b'}'), _, _) if format => todo!(),  // an error
                (Some(b'"'), _, _) if quote_count == 1 => {
                    // TODO: make sure that it's a valid utf-8
                    let interned = intern_string(&self.buffer1, &self.intermediate_dir).unwrap();

                    if format {
                        self.fstring_buffer.push(TokensOrString::String(interned));
                        self.tokens.push(Token {
                            kind: TokenKind::FormattedString {
                                raw: true,
                                elements: self.fstring_buffer.drain(..).filter(
                                    |t| match t {
                                        TokensOrString::String(s) if s.len() == 0 => false,
                                        _ => true,
                                    }
                                ).collect(),
                            },
                            span: Span::range(
                                self.file,
                                self.token_start,
                                self.cursor,
                            ),
                        });
                    }

                    else {
                        self.tokens.push(Token {
                            kind: TokenKind::String {
                                binary,
                                raw: true,
                                regex,
                                s: interned,
                            },
                            span: Span::range(
                                self.file,
                                self.token_start,
                                self.cursor,
                            ),
                        });
                    }

                    self.state = LexState::Init;
                    self.cursor += 1;
                },
                (Some(b'"'), Some(b'"'), Some(b'"')) => {
                    if quote_count == 3 {
                        todo!()
                    }

                    else {
                        let curr_quote_count = count_quotes(&self.input_bytes, self.cursor).unwrap_or(256);

                        if curr_quote_count >= quote_count {
                            todo!()
                        }

                        else {
                            self.buffer1.push(b'"');
                            self.buffer1.push(b'"');
                            self.buffer1.push(b'"');
                            self.cursor += 3;
                        }
                    }
                },
                (Some(x), _, _) => {
                    self.buffer1.push(*x);
                    self.cursor += 1;
                },
                (None, _, _) => {
                    return Err(Error {
                        kind: ErrorKind::UnterminatedStringLiteral,
                        spans: Span::range(
                            self.file,
                            self.token_start,
                            self.token_start + quote_count,
                        ).simple_error(),
                        note: None,
                    });
                },
            },
            LexState::String { format, binary, raw: false, quote_count, regex } => match (
                self.input_bytes.get(self.cursor),
                self.input_bytes.get(self.cursor + 1),
                self.input_bytes.get(self.cursor + 2),
                self.input_bytes.get(self.cursor + 3),
            ) {
                (Some(b'{'), Some(b'{'), _, _) if format => {
                    self.buffer1.push(b'{');
                    self.cursor += 2;
                },
                (Some(b'}'), Some(b'}'), _, _) if format => {
                    self.buffer1.push(b'}');
                    self.cursor += 2;
                },
                (Some(b'{'), _, _, _) if format => {
                    let interned = intern_string(&self.buffer1, &self.intermediate_dir).unwrap();
                    self.fstring_buffer.push(TokensOrString::String(interned));
                    self.lex_formatted_string()?;
                    self.buffer1.clear();
                },
                (Some(b'}'), _, _, _) if format => todo!(),  // an error
                // valid escape
                (Some(b'\\'), Some(y @ (b'\'' | b'"' | b'\\' | b'n' | b'r' | b't' | b'0')), _, _) => {
                    let byte = match *y {
                        b'\'' | b'"' | b'\\' => *y,
                        b'n' => b'\n',
                        b'r' => b'\r',
                        b't' => b'\t',
                        b'0' => b'\0',
                        _ => unreachable!(),
                    };
                    self.buffer1.push(byte);
                    self.cursor += 2;
                },
                // ascii escape
                (Some(b'\\'), Some(b'x'), Some(z @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')), Some(w @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F'))) => todo!(),
                // TODO: unicode escape
                // invalid escape
                (Some(b'\\'), Some(_), _, _) => {
                    return Err(Error {
                        kind: ErrorKind::InvalidEscape,
                        spans: Span::range(
                            self.file,
                            self.cursor + 1,
                            self.cursor + 2,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
                (Some(b'"'), _, _, _) if quote_count == 1 => {
                    let interned = intern_string(&self.buffer1, &self.intermediate_dir).unwrap();

                    if format {
                        self.fstring_buffer.push(TokensOrString::String(interned));
                        self.tokens.push(Token {
                            kind: TokenKind::FormattedString {
                                raw: false,
                                elements: self.fstring_buffer.drain(..).filter(
                                    |t| match t {
                                        TokensOrString::String(s) if s.len() == 0 => false,
                                        _ => true,
                                    }
                                ).collect(),
                            },
                            span: Span::range(
                                self.file,
                                self.token_start,
                                self.cursor,
                            ),
                        });
                    }

                    else {
                        self.tokens.push(Token {
                            kind: TokenKind::String {
                                binary,
                                raw: false,
                                regex,
                                s: interned,
                            },
                            span: Span::range(
                                self.file,
                                self.token_start,
                                self.cursor + 1,
                            ),
                        });
                    }

                    self.state = LexState::Init;
                    self.cursor += 1;
                },
                (Some(b'"'), Some(b'"'), Some(b'"'), _) if quote_count >= 3 => todo!(),
                // valid char (utf-8)
                (Some(x @ 0..=127), _, _, _) => {
                    self.buffer1.push(*x);
                    self.cursor += 1;
                },
                (Some(x @ 192..=223), Some(y @ 128..=191), _, _) => {
                    self.buffer1.push(*x);
                    self.buffer1.push(*y);
                    self.cursor += 2;
                },
                (Some(x @ 224..=239), Some(y @ 128..=191), Some(z @ 128..=191), _) => {
                    self.buffer1.push(*x);
                    self.buffer1.push(*y);
                    self.buffer1.push(*z);
                    self.cursor += 3;
                },
                (Some(x @ 240..=247), Some(y @ 128..=191), Some(z @ 128..=191), Some(w @ 128..=191)) => {
                    self.buffer1.push(*x);
                    self.buffer1.push(*y);
                    self.buffer1.push(*z);
                    self.buffer1.push(*w);
                    self.cursor += 4;
                },
                (Some(_), _, _, _) => {
                    return Err(Error {
                        kind: ErrorKind::InvalidUtf8,
                        spans: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 1,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
                (None, _, _, _) => {
                    return Err(Error {
                        kind: ErrorKind::UnterminatedStringLiteral,
                        spans: Span::range(
                            self.file,
                            self.token_start,
                            self.token_start + quote_count,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
            },
            // NOTE: empty char literals are already filtered out!
            // NOTE: the cursor is pointing at the first byte of the content (not the quote)
            LexState::Char { binary } => match (
                self.input_bytes.get(self.cursor),
                self.input_bytes.get(self.cursor + 1),
                self.input_bytes.get(self.cursor + 2),
                self.input_bytes.get(self.cursor + 3),
                self.input_bytes.get(self.cursor + 4),
            ) {
                // valid escape
                (Some(b'\\'), Some(y @ (b'\'' | b'"' | b'\\' | b'n' | b'r' | b't' | b'0')), Some(b'\''), _, _) => {
                    let (ch, b) = match *y {
                        b'\'' => ('\'', b'\''),
                        b'"' => ('"', b'"'),
                        b'\\' => ('\\', b'\\'),
                        b'n' => ('\n', b'\n'),
                        b'r' => ('\r', b'\r'),
                        b't' => ('\t', b'\t'),
                        b'0' => ('\0', b'\0'),
                        _ => unreachable!(),
                    };

                    // It's always ascii, so we don't have to check that.
                    if binary {
                        self.tokens.push(Token {
                            kind: TokenKind::Byte(b),
                            span: Span::range(
                                self.file,
                                self.token_start,
                                self.cursor + 3,
                            ),
                        });
                    }

                    else {
                        self.tokens.push(Token {
                            kind: TokenKind::Char(ch as u32),
                            span: Span::range(
                                self.file,
                                self.token_start,
                                self.cursor + 3,
                            ),
                        });
                    }

                    self.state = LexState::Init;
                    self.cursor += 3;
                },
                // ascii escape
                // Well, it can exceed the ascii range, ... but who cares?
                (Some(b'\\'), Some(b'x'), Some(z @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')), Some(w @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')), Some(b'\'')) => {
                    let n1 = match *z {
                        b'0'..=b'9' => z - b'0',
                        b'a'..=b'f' => z - b'a' + 10,
                        b'A'..=b'F' => z - b'A' + 10,
                        _ => unreachable!(),
                    } as u32;
                    let n2 = match *w {
                        b'0'..=b'9' => w - b'0',
                        b'a'..=b'f' => w - b'a' + 10,
                        b'A'..=b'F' => w - b'A' + 10,
                        _ => unreachable!(),
                    } as u32;

                    if binary {
                        if n1 < 8 {
                            self.tokens.push(Token {
                                kind: TokenKind::Byte((n1 * 16 + n2) as u8),
                                span: Span::range(
                                    self.file,
                                    self.token_start,
                                    self.cursor + 5,
                                ),
                            });
                        }

                        else {
                            return Err(Error {
                                kind: ErrorKind::InvalidByteLiteral,
                                spans: Span::range(
                                    self.file,
                                    self.token_start,
                                    self.cursor + 5,
                                ).simple_error(),
                                note: Some(format!("A byte char literal must be an ascii char. Perhaps you mean `#{}`?", n1 * 16 + n2)),
                            });
                        }
                    }

                    else {
                        self.tokens.push(Token {
                            kind: TokenKind::Char(n1 * 16 + n2),
                            span: Span::range(
                                self.file,
                                self.token_start,
                                self.cursor + 5,
                            ),
                        });
                    }

                    self.state = LexState::Init;
                    self.cursor += 5;
                },
                (Some(b'\\'), Some(b'u'), Some(b'{'), _, _) => {
                    let escape_start = self.cursor;
                    self.cursor += 3;
                    let mut n = 0;

                    loop {
                        match self.input_bytes.get(self.cursor) {
                            Some(x @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')) => {
                                let x = match *x {
                                    b'0'..=b'9' => x - b'0',
                                    b'a'..=b'f' => x - b'a' + 10,
                                    b'A'..=b'F' => x - b'A' + 10,
                                    _ => unreachable!(),
                                } as u32;

                                n <<= 4;
                                n |= x;
                                self.cursor += 1;

                                if n > 0x10ffff {
                                    return Err(Error {
                                        kind: ErrorKind::InvalidUnicodeCharacter,
                                        spans: Span::range(
                                            self.file,
                                            escape_start,
                                            escape_start + 1,
                                        ).simple_error(),
                                        ..Error::default()
                                    });
                                }
                            },
                            Some(b'}') => {
                                self.cursor += 1;
                                break;
                            },
                            Some(_) => {
                                return Err(Error {
                                    kind: ErrorKind::InvalidUnicodeEscape,
                                    spans: Span::range(
                                        self.file,
                                        self.cursor,
                                        self.cursor + 1,
                                    ).simple_error(),
                                    ..Error::default()
                                });
                            },
                            None => {
                                return Err(Error {
                                    kind: ErrorKind::UnclosedDelimiter(b'}'),
                                    spans: Span::eof(self.file).simple_error(),
                                    ..Error::default()
                                });
                            },
                        }
                    }

                    match self.input_bytes.get(self.cursor) {
                        Some(b'\'') => {
                            self.cursor += 1;
                        },
                        Some(_) => todo!(),
                        None => todo!(),
                    }

                    self.state = LexState::Init;

                    if binary {
                        if n < 128 {
                            self.tokens.push(Token {
                                kind: TokenKind::Byte(n as u8),
                                span: Span::range(
                                    self.file,
                                    self.token_start,
                                    self.cursor,
                                ),
                            });
                        }

                        else {
                            let error_note = if n < 256 {
                                format!("A byte char literal must be an ascii char. Perhaps you mean `#{n}`?")
                            } else {
                                String::from("A byte must be in range #0..=#255.")
                            };

                            return Err(Error {
                                kind: ErrorKind::InvalidByteLiteral,
                                spans: Span::range(
                                    self.file,
                                    self.token_start,
                                    self.cursor,
                                ).simple_error(),
                                note: Some(error_note),
                            });
                        }
                    }

                    else {
                        self.tokens.push(Token {
                            kind: TokenKind::Char(n),
                            span: Span::range(
                                self.file,
                                self.token_start,
                                self.cursor,
                            ),
                        });
                    }
                },
                (Some(b'\\'), Some(_), _, _, _) => {
                    return Err(Error {
                        kind: ErrorKind::InvalidEscape,
                        spans: Span::range(
                            self.file,
                            self.cursor + 1,
                            self.cursor + 2,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
                (Some(b'\r' | b'\n' | b'\t' | b'\''), _, _, _, _) => {
                    return Err(Error {
                        kind: ErrorKind::InvalidCharLiteral,
                        spans: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 1,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
                // valid char (utf-8)
                (Some(0..=127), Some(b'\''), _, _, _) |
                (Some(192..=223), Some(128..=191), Some(b'\''), _, _) |
                (Some(224..=239), Some(128..=191), Some(128..=191), Some(b'\''), _) |
                (Some(240..=247), Some(128..=191), Some(128..=191), Some(128..=191), Some(b'\'')) => {
                    let (n, l) = match (
                        self.input_bytes.get(self.cursor),
                        self.input_bytes.get(self.cursor + 1),
                        self.input_bytes.get(self.cursor + 2),
                        self.input_bytes.get(self.cursor + 3),
                    ) {
                        (Some(x @ 0..=127), _, _, _) => (
                            *x as u32,
                            1,
                        ),
                        (Some(x @ 192..=223), Some(y @ 128..=191), _, _) => (
                            ((*x as u32 - 192) << 6) | (*y as u32 - 128),
                            2,
                        ),
                        (Some(x @ 224..=239), Some(y @ 128..=191), Some(z @ 128..=191), _) => (
                            ((*x as u32 - 224) << 12) | ((*y as u32 - 128) << 6) | (*z as u32 - 128),
                            3,
                        ),
                        (Some(x @ 240..=247), Some(y @ 128..=191), Some(z @ 128..=191), Some(w @ 128..=191)) => (
                            ((*x as u32 - 240) << 18) | ((*y as u32 - 128) << 12) | ((*z as u32 - 128) << 6) | (*w as u32 - 128),
                            4,
                        ),
                        _ => unreachable!(),
                    };

                    match char::from_u32(n) {
                        Some(_) => {
                            if binary {
                                if n < 128 {
                                    self.tokens.push(Token {
                                        kind: TokenKind::Byte(n as u8),
                                        span: Span::range(
                                            self.file,
                                            self.token_start,
                                            self.cursor + l + 1,
                                        ),
                                    });
                                }

                                else {
                                    let error_note = if n < 256 {
                                        format!("A byte char literal must be an ascii char. Perhaps you mean `#{n}`?")
                                    } else {
                                        String::from("A byte must be in range #0..=#255.")
                                    };

                                    return Err(Error {
                                        kind: ErrorKind::InvalidByteLiteral,
                                        spans: Span::range(
                                            self.file,
                                            self.token_start,
                                            self.cursor + l + 1,
                                        ).simple_error(),
                                        note: Some(error_note),
                                    });
                                }
                            }

                            else {
                                self.tokens.push(Token {
                                    kind: TokenKind::Char(n),
                                    span: Span::range(
                                        self.file,
                                        self.token_start,
                                        self.cursor + l + 1,
                                    ),
                                });
                            }

                            self.state = LexState::Init;
                            self.cursor += l + 1;
                        },
                        None => {
                            return Err(Error {
                                kind: ErrorKind::InvalidUtf8,
                                // It points to the quote character because it doesn't know which byte is erroneous.
                                spans: Span::range(
                                    self.file,
                                    self.cursor,
                                    self.cursor + 1,
                                ).simple_error(),
                                ..Error::default()
                            });
                        },
                    }
                },
                // invalid utf-8
                (Some(128..), _, _, _, _) => {
                    return Err(Error {
                        kind: ErrorKind::InvalidUtf8,
                        spans: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 1,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
                // etc error (probably multi-character literal)
                (Some(_), _, _, _, _) => {
                    return Err(Error {
                        kind: ErrorKind::InvalidCharLiteral,
                        spans: Span::range(
                            self.file,
                            self.token_start,
                            self.token_start + 1,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
                (None, _, _, _, _) => {
                    return Err(Error {
                        kind: ErrorKind::UnterminatedCharLiteral,
                        spans: Span::range(
                            self.file,
                            self.token_start,
                            self.token_start + 1,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
            },
            LexState::Ident => match (
                self.input_bytes.get(self.cursor),
                self.input_bytes.get(self.cursor + 1),
                self.input_bytes.get(self.cursor + 2),
                self.input_bytes.get(self.cursor + 3),
            ) {
                (Some(x @ (b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_')), _, _, _) => {
                    self.buffer1.push(*x);
                    self.cursor += 1;
                },
                (Some(x @ 192..=223), Some(y @ 128..=191), _, _) => {
                    self.buffer1.push(*x);
                    self.buffer1.push(*y);
                    self.cursor += 2;
                },
                (Some(x @ 224..=239), Some(y @ 128..=191), Some(z @ 128..=191), _) => {
                    self.buffer1.push(*x);
                    self.buffer1.push(*y);
                    self.buffer1.push(*z);
                    self.cursor += 3;
                },
                (Some(x @ 240..=247), Some(y @ 128..=191), Some(z @ 128..=191), Some(w @ 128..=191)) => {
                    self.buffer1.push(*x);
                    self.buffer1.push(*y);
                    self.buffer1.push(*z);
                    self.buffer1.push(*w);
                    self.cursor += 4;
                },
                (Some(128..), _, _, _) => {
                    return Err(Error {
                        kind: ErrorKind::InvalidUtf8,
                        spans: Span::range(
                            self.file,
                            self.cursor,
                            self.cursor + 1,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
                _ => {
                    let token_kind = match self.buffer1.as_slice() {
                        b"as" => TokenKind::Keyword(Keyword::As),
                        b"assert" => TokenKind::Keyword(Keyword::Assert),
                        b"else" => TokenKind::Keyword(Keyword::Else),
                        b"enum" => TokenKind::Keyword(Keyword::Enum),
                        b"fn" => TokenKind::Keyword(Keyword::Fn),
                        b"if" => TokenKind::Keyword(Keyword::If),
                        b"let" => TokenKind::Keyword(Keyword::Let),
                        b"match" => TokenKind::Keyword(Keyword::Match),
                        b"mod" => TokenKind::Keyword(Keyword::Mod),
                        b"pub" => TokenKind::Keyword(Keyword::Pub),
                        b"struct" => TokenKind::Keyword(Keyword::Struct),
                        b"type" => TokenKind::Keyword(Keyword::Type),
                        b"use" => TokenKind::Keyword(Keyword::Use),
                        _ => {
                            // Lexer already checked that it's a valid utf8.
                            let identifier = String::from_utf8_lossy(self.buffer1.as_slice());

                            for ch in identifier.chars() {
                                match ch {
                                    // Allowed characters in an identifer
                                    // ascii
                                    '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' |
                                    // CJK
                                    '㐀'..='鿿' | 'ぁ'..='ゖ' | 'ァ'..='ヺ' | '가'..='힣' |
                                    // https://www.unicode.org/Public/emoji/1.0//emoji-data.txt
                                    '☀'..='❤' | '🌀'..='🙏' | '🚀'..='🛼' | '🤌'..='🫸' => {},
                                    _ => {
                                        return Err(Error {
                                            kind: ErrorKind::InvalidCharacterInIdent(ch),

                                            // It'd be lovely to calc the exact span of the character, but I'm too lazy to do that.
                                            spans: Span::range(
                                                self.file,
                                                self.token_start,
                                                self.token_start + 1,
                                            ).simple_error(),
                                            ..Error::default()
                                        });
                                    },
                                }
                            }

                            let interned = intern_string(&self.buffer1, &self.intermediate_dir).unwrap();
                            TokenKind::Ident(interned)
                        },
                    };

                    self.tokens.push(Token {
                        kind: token_kind,
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.cursor,
                        ),
                    });
                    self.state = LexState::Init;
                },
            },
            LexState::FieldModifier => match self.input_bytes.get(self.cursor) {
                Some(x @ (b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_')) => {
                    self.buffer1.push(*x);
                    self.cursor += 1;
                },
                _ => {
                    let interned = intern_string(&self.buffer1, &self.intermediate_dir).unwrap();

                    self.tokens.push(Token {
                        kind: TokenKind::FieldModifier(interned),
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.cursor,
                        ),
                    });
                    self.state = LexState::Init;
                },
            },
            LexState::Integer(base) => match (
                self.input_bytes.get(self.cursor),
                self.input_bytes.get(self.cursor + 1),
                self.input_bytes.get(self.cursor + 2),
            ) {
                // 123e4 == 1230000.0
                (Some(b'e' | b'E'), _, _) if base == Base::Decimal => {
                    self.cursor += 1;
                    self.state = LexState::Exp;
                },

                // `b'g'..=b'z'` is always error, but it matches the
                // range so that it can generate a better error message
                (Some(x @ (b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z')), _, _) => {
                    if !base.is_valid_digit(*x) {
                        return Err(Error {
                            kind: ErrorKind::InvalidNumberLiteral,
                            spans: Span::range(
                                self.file,
                                self.cursor,
                                self.cursor + 1,
                            ).simple_error(),
                            note: Some(base.invalid_digit_error_message(*x)),
                        });
                    }

                    self.buffer1.push(*x);
                    self.cursor += 1;
                },
                (Some(b'_'), _, _) => {
                    self.cursor += 1;
                },
                // perhaps the user intends decimal point (the first dot) + range (2nd and 3rd dots),
                // but it's an ambiguous syntax, so we just reject this
                (Some(b'.'), Some(b'.'), Some(b'.')) => {
                    // I want to tell the user that it's an ambiuous syntax, but there's no error variant for that
                    todo!()
                },

                // `64..` is a range pattern/expr. The dots are not decimal points!
                (Some(b'.'), Some(b'.'), _) => {
                    let interned = intern_number(base, &self.buffer1, &self.buffer2, 0, true /* is_integer */);

                    self.tokens.push(Token {
                        kind: TokenKind::Number(interned),
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.cursor,
                        ),
                    });
                    self.state = LexState::Init;
                },
                (Some(b'.'), _, _) => match base {
                    Base::Decimal => {
                        self.buffer2.clear();
                        self.state = LexState::Fraction;
                        self.cursor += 1;
                    },
                    Base::Hexadecimal | Base::Octal | Base::Binary => {
                        return Err(Error {
                            kind: ErrorKind::InvalidNumberLiteral,
                            spans: Span::range(
                                self.file,
                                self.cursor,
                                self.cursor + 1,
                            ).simple_error(),
                            ..Error::default()
                        });
                    },
                },
                _ => {
                    let interned = intern_number(base, &self.buffer1, &self.buffer2, 0, true /* is_integer */);

                    self.tokens.push(Token {
                        kind: TokenKind::Number(interned),
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.cursor,
                        ),
                    });
                    self.state = LexState::Init;
                },
            },
            LexState::Fraction => match self.input_bytes.get(self.cursor) {
                Some(x @ (b'0'..=b'9')) => {
                    self.buffer2.push(*x);
                    self.cursor += 1;
                },
                Some(b'_') => {
                    self.cursor += 1;
                },
                Some(b'e' | b'E') => {
                    self.cursor += 1;
                    self.state = LexState::Exp;
                },
                Some(_) | None => {
                    // At this point, `Base` must be Decimal. (otherwise lex error)
                    let interned = intern_number(Base::Decimal, &self.buffer1, &self.buffer2, 0, false /* is_integer */);

                    self.tokens.push(Token {
                        kind: TokenKind::Number(interned),
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.cursor,
                        ),
                    });
                    self.state = LexState::Init;
                },
            },
            LexState::Exp => {
                let is_neg = if let Some(b'-') = self.input_bytes.get(self.cursor) {
                    self.cursor += 1;
                    true
                } else {
                    false
                };

                // There're only two buffers in the session :(
                let mut e_buffer = vec![];

                loop {
                    match self.input_bytes.get(self.cursor) {
                        Some(x @ b'0'..=b'9') => {
                            e_buffer.push(*x);
                            self.cursor += 1;
                        },
                        _ => {
                            break;
                        },
                    }
                }

                let mut exp: i64 = match String::from_utf8(e_buffer).unwrap().parse() {
                    Ok(n) => n,
                    Err(e) => match e.kind() {
                        IntErrorKind::Empty => todo!(),
                        IntErrorKind::PosOverflow => todo!(),
                        _ => unreachable!(),
                    },
                };

                if is_neg {
                    match exp.checked_neg() {
                        Some(n) => {
                            exp = n;
                        },
                        None => todo!(),
                    }
                }

                // At this point, `Base` must be Decimal. (otherwise lex error)
                let interned = intern_number(Base::Decimal, &self.buffer1, &self.buffer2, exp, false /* is_integer */);

                self.tokens.push(Token {
                    kind: TokenKind::Number(interned),
                    span: Span::range(
                        self.file,
                        self.token_start,
                        self.cursor,
                    ),
                });
                self.state = LexState::Init;
            },
            LexState::LineComment => match self.input_bytes.get(self.cursor) {
                Some(b'\n') => {
                    self.state = LexState::Init;
                    self.cursor += 1;
                },
                Some(_) => {
                    self.cursor += 1;
                },
                None => {
                    self.state = LexState::Init;
                },
            },
            LexState::DocComment { top_level } => match self.input_bytes.get(self.cursor) {
                Some(b'\n') => {
                    let interned = intern_string(&self.buffer1, &self.intermediate_dir).unwrap();

                    self.tokens.push(Token {
                        kind: TokenKind::DocComment {
                            top_level,
                            doc: interned,
                        },
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.cursor,
                        ),
                    });
                    self.state = LexState::Init;
                    self.cursor += 1;
                },
                Some(x) => {
                    self.buffer1.push(*x);
                    self.cursor += 1;
                },
                // TODO: I don't like this implementation
                //       In this case, the DocComment itself is valid, but it's an error because the
                //       DocComment is not attached to anything.
                //       My original idea was "lexer should guarantee that there's no dangling DocComment at the end",
                //       but the lexer shouldn't throw this kind of error.
                None => {
                    return Err(Error {
                        kind: ErrorKind::UnexpectedEof {
                            expected: ErrorToken::Declaration,
                        },
                        spans: Span::eof(self.file).simple_error(),
                        ..Error::default()
                    });
                },
            },
            LexState::BlockComment => match (self.input_bytes.get(self.cursor), self.input_bytes.get(self.cursor + 1)) {
                (Some(b'*'), Some(b'/')) => {
                    self.state = LexState::Init;
                    self.cursor += 2;
                },
                (Some(_), _) => {
                    self.cursor += 1;
                },
                (None, _) => {
                    return Err(Error {
                        kind: ErrorKind::UnterminatedBlockComment,

                        // opening of the block comment
                        spans: Span::range(
                            self.file,
                            self.token_start,
                            self.token_start + 2,
                        ).simple_error(),
                        ..Error::default()
                    });
                },
            },
        }

        Ok(false)
    }

    fn lex_formatted_string(&mut self) -> Result<(), Error> {
        assert!(matches!(self.input_bytes.get(self.cursor), Some(b'{')));
        let mut value_end = 0;

        for i in (self.cursor + 1).. {
            match self.input_bytes.get(i) {
                Some(x @ (b'{' | b'\\' | b'"')) => {
                    return Err(Error {
                        kind: ErrorKind::NotAllowedCharInFString(*x),
                        spans: Span::range(
                            self.file,
                            i,
                            i + 1,
                        ).simple_error(),
                        note: None,
                    });
                },
                Some(b'}') => {
                    value_end = i;
                    break;
                },
                Some(_) => {},
                None => {
                    return Err(Error {
                        kind: ErrorKind::UnterminatedStringLiteral,
                        spans: Span::range(
                            self.file,
                            self.token_start,
                            self.token_start + 1,
                        ).simple_error(),
                        note: None,
                    });
                },
            }
        }

        let mut tmp_session = lex(
            self.file,
            self.input_bytes[(self.cursor + 1)..value_end].to_vec(),
            self.intermediate_dir.clone(),
            self.is_std,
        );
        tmp_session.offset_spans(self.cursor + 1);

        if !tmp_session.errors.is_empty() {
            return Err(tmp_session.errors[0].clone());
        }

        if tmp_session.tokens.is_empty() {
            // an empty value in fstring is an error
            todo!()
        }

        self.fstring_buffer.push(TokensOrString::Tokens{
            tokens: tmp_session.tokens,
            span: Span::range(
                self.file,
                self.cursor,
                value_end,
            ),
        });

        // points to the byte after '}'
        self.cursor = value_end + 1;
        Ok(())
    }

    fn group_tokens(&mut self) {
        self.tokens = group_tokens_recursive(&self.tokens);
    }

    fn offset_spans(&mut self, offset: usize) {
        for token in self.tokens.iter_mut() {
            token.offset_span(offset);
        }

        for error in self.errors.iter_mut().chain(self.warnings.iter_mut()) {
            for span in error.spans.iter_mut() {
                span.span.offset(offset);
            }
        }
    }
}

// There's a historical reason why it doesn't allow using more than 255 quotes.
// The reason is irrelevant now, but I like it filtering too many quotes.
fn count_quotes(buffer: &[u8], mut cursor: usize) -> Result<usize, ()> {
    let mut count = 0;

    loop {
        match buffer.get(cursor) {
            Some(b'"') => {
                count += 1;
                cursor += 1;

                if count == 256 {
                    return Err(());
                }
            },
            Some(_) => {
                return Ok(count);
            },
            None => {
                return Ok(count);
            },
        }
    }
}

// It assumes that there's no syntax error.
fn group_tokens_recursive(tokens: &[Token]) -> Vec<Token> {
    let mut result = Vec::with_capacity(tokens.len());
    let mut i = 0;

    loop {
        match tokens.get(i) {
            Some(Token {
                kind: TokenKind::GroupDelim { delim, id },
                span: opening_span,
            }) => {
                let delim = delim.unwrap();
                let mut has_inner_group = false;

                for j in (i + 1).. {
                    if let TokenKind::GroupDelim { id: id_, .. } = &tokens[j].kind {
                        if id == id_ {
                            let mut inner_tokens = tokens[(i + 1)..j].to_vec();

                            if has_inner_group {
                                inner_tokens = group_tokens_recursive(&inner_tokens);
                            }

                            result.push(Token {
                                kind: TokenKind::Group {
                                    delim,
                                    tokens: inner_tokens,
                                },
                                span: opening_span.merge(tokens[j].span),
                            });
                            i = j + 1;
                            break;
                        }

                        else {
                            has_inner_group = true;
                        }
                    }
                }
            },
            Some(t) => {
                result.push(t.clone());
                i += 1;
            },
            None => {
                break;
            },
        }
    }

    result
}
