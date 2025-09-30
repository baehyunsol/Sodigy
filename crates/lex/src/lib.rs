use sodigy_error::{Error, ErrorKind};
use sodigy_file::File;
use sodigy_keyword::Keyword;
use sodigy_number::{Base, InternedNumber, intern_number};
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};
use sodigy_token::{Delim, ErrorToken, Punct, Token, TokenKind};
use std::collections::hash_map::{Entry, HashMap};

pub struct Session {
    pub file: File,
    input_buffer: Vec<u8>,
    state: LexState,
    cursor: usize,
    offset: usize,
    pub tokens: Vec<Token>,
    string_map: HashMap<InternedString, Vec<u8>>,
    pub errors: Vec<Error>,

    group_stack: Vec<(u8, Span)>,  // u8: b']' | b'}' | b')', Span: span of the opening delim

    // cursor + offset of the start of the current token
    token_start: usize,

    // identifier, integer
    buffer1: Vec<u8>,

    // fraction
    buffer2: Vec<u8>,

    // Even though there's an error, it might try to lex further, so that it can find more errors.
    // If it encounters a non-continuable error, it immediately sets `halt_with_error` and halts.
    halt_with_error: bool,
    halt_without_error: bool,
}

#[derive(Debug)]
enum LexState {
    Init,

    // `StringPrefix` first parses prefix `b`, `f` or `r` before the literal
    // then `StringInit` counts the number of double quote characters
    // then `String` parses the content of the literal
    StringPrefix,
    StringInit {
        binary: bool,
        format: bool,
        raw: bool,
    },
    String {
        binary: bool,
        raw: bool,
        quote_count: usize,
    },
    Char {
        binary: bool,
    },
    FormattedString {
        raw: bool,
        quote_count: usize,
    },

    Identifier,
    FieldModifier,
    Decorator,
    Integer(Base),
    Fraction,
    LineComment,
    DocComment,
    BlockComment,
}

pub fn lex_gara(input: Vec<u8>) -> Result<Vec<Token>, Vec<Error>> {
    let mut gara_session = Session {
        file: File::gara(),
        input_buffer: input,
        state: LexState::Init,
        cursor: 0,
        offset: 0,
        tokens: vec![],
        string_map: HashMap::new(),
        errors: vec![],
        group_stack: vec![],
        token_start: 0,
        buffer1: vec![],
        buffer2: vec![],
        halt_with_error: false,
        halt_without_error: false,
    };

    while !gara_session.halt_with_error && !gara_session.halt_without_error {
        gara_session.step();
    }

    if gara_session.errors.is_empty() {
        gara_session.group_tokens();
        gara_session.merge_doc_comments();
        Ok(gara_session.tokens)
    }

    else {
        Err(gara_session.errors)
    }
}

impl Session {
    fn step(&mut self) {
        if let Err(e) = self.try_load_input() {
            self.errors.push(e);
            self.halt_with_error = true;
            return;
        }

        if self.halt_with_error || self.halt_without_error {
            return;
        }

        match self.state {
            LexState::Init => match (self.input_buffer.get(self.cursor), self.input_buffer.get(self.cursor + 1), self.input_buffer.get(self.cursor + 2)) {
                (Some(b'a'..=b'z'), Some(b'a'..=b'z'), Some(b'"' | b'\'')) |
                (Some(b'a'..=b'z'), Some(b'"' | b'\''), _) |
                (Some(b'"' | b'\''), _, _) => {
                    self.state = LexState::StringPrefix;
                },
                (Some(x @ (b'a'..=b'z' | b'A'..=b'Z' | b'_')), _, _) => {
                    self.buffer1.clear();
                    self.buffer1.push(*x);

                    self.token_start = self.cursor + self.offset;
                    self.state = LexState::Identifier;
                    self.cursor += 1;
                },
                (Some(b'`'), Some(y @ (b'a'..=b'z' | b'A'..=b'Z' | b'_')), _) => {
                    self.buffer1.clear();
                    self.buffer1.push(*y);

                    self.token_start = self.cursor + self.offset;
                    self.state = LexState::FieldModifier;
                    self.cursor += 2;
                },
                (Some(b'@'), Some(y @ (b'a'..=b'z' | b'A'..=b'Z' | b'_')), _) => {
                    self.buffer1.clear();
                    self.buffer1.push(*y);

                    self.token_start = self.cursor + self.offset;
                    self.state = LexState::Decorator;
                    self.cursor += 2;
                },
                (Some(b'0'), Some(b'x' | b'X' | b'o' | b'O' | b'b' | b'B'), _) => todo!(),
                (Some(b'0'..=b'9'), Some(b'a'..=b'z' | b'A'..=b'Z' | b'_'), _) => {
                    self.errors.push(Error {
                        kind: ErrorKind::InvalidNumberLiteral,
                        span: Span::range(
                            self.file,
                            self.cursor + 1 + self.offset,
                            self.cursor + 2 + self.offset,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
                (Some(b'0'), Some(b'.'), _) => {
                    self.buffer1.clear();
                    self.buffer1.push(b'0');
                    self.buffer2.clear();
                    self.token_start = self.cursor + self.offset;
                    self.state = LexState::Fraction;
                    self.cursor += 2;
                },
                (Some(b'0'), _, _) => {
                    self.tokens.push(Token {
                        kind: TokenKind::Number(InternedNumber::zero()),
                        span: Span::range(
                            self.file,
                            self.cursor + self.offset,
                            self.cursor + 1 + self.offset,
                        ),
                    });
                    self.cursor += 1;
                },
                (Some(x @ (b'1'..=b'9')), _, _) => {
                    self.buffer1.clear();
                    self.buffer1.push(*x);

                    self.token_start = self.cursor + self.offset;
                    self.state = LexState::Integer(Base::Decimal);
                    self.cursor += 1;
                },
                (Some(b'/'), Some(b'/'), Some(b'/')) => {
                    self.token_start = self.cursor + self.offset;
                    self.state = LexState::DocComment;
                    self.cursor += 3;
                },
                (Some(b'/'), Some(b'/'), _) => {
                    self.state = LexState::LineComment;
                    self.cursor += 2;
                },
                (Some(b'/'), Some(b'*'), _) => {
                    self.token_start = self.cursor + self.offset;
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
                        self.cursor + self.offset,
                        self.cursor + 1 + self.offset,
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
                        self.cursor + self.offset,
                        self.cursor + 2 + self.offset,
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
                                self.cursor + self.offset,
                                self.cursor + 1 + self.offset,
                            ),
                        });
                        self.cursor += 1;
                    },
                    Some((delim, _)) => {
                        self.errors.push(Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: ErrorToken::Character(delim),
                                got: ErrorToken::Character(*x),
                            },
                            span: Span::range(
                                self.file,
                                self.cursor + self.offset,
                                self.cursor + 1 + self.offset,
                            ),
                            ..Error::default()
                        });
                        self.halt_with_error = true;
                    },
                    None => {
                        self.errors.push(Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: ErrorToken::Any,
                                got: ErrorToken::Character(*x),
                            },
                            span: Span::eof(self.file),
                            ..Error::default()
                        });
                        self.halt_with_error = true;
                    },
                },
                (Some(x @ (b'!' | b'.' | b'<' | b'=' | b'>')), Some(y @ (b'.' | b'<' | b'=' | b'>')), _) => {
                    let punct = match (x, y) {
                        (b'!', b'=') => Some(Punct::Neq),
                        (b'.', b'.') => Some(Punct::DotDot),
                        (b'<', b'<') => Some(Punct::Shl),
                        (b'<', b'=') => Some(Punct::Leq),
                        (b'=', b'=') => Some(Punct::Eq),
                        (b'=', b'>') => Some(Punct::Arrow),
                        (b'>', b'=') => Some(Punct::Geq),
                        (b'>', b'>') => Some(Punct::Shr),
                        _ => None,
                    };

                    match punct {
                        Some(p) => {
                            self.tokens.push(Token {
                                kind: TokenKind::Punct(p),
                                span: Span::range(
                                    self.file,
                                    self.cursor + self.offset,
                                    self.cursor + 2 + self.offset,
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
                                    self.cursor + self.offset,
                                    self.cursor + 1 + self.offset,
                                ),
                            });
                            self.tokens.push(Token {
                                kind: TokenKind::Punct((*y).into()),
                                span: Span::range(
                                    self.file,
                                    self.cursor + 1 + self.offset,
                                    self.cursor + 2 + self.offset,
                                ),
                            });
                            self.cursor += 2;
                        },
                    }
                },
                (Some(x @ (
                    b'!' | b'#' | b'$' | b'%' | b'&' |
                    b'*' | b'+' | b',' | b'-' | b'.' |
                    b'/' | b':' | b';' | b'<' | b'=' |
                    b'>' | b'?' | b'@' | b'^' | b'~'
                )), _, _) => {
                    self.tokens.push(Token {
                        kind: TokenKind::Punct((*x).into()),
                        span: Span::range(
                            self.file,
                            self.cursor + self.offset,
                            self.cursor + 1 + self.offset,
                        ),
                    });
                    self.cursor += 1;
                },
                (Some(x), _, _) => panic!("TODO: {:?}", *x as char),
                (None, _, _) => {
                    if let Some((delim, span)) = self.group_stack.pop() {
                        self.errors.push(Error {
                            kind: ErrorKind::UnclosedDelimiter(delim),
                            span: span,
                            ..Error::default()
                        });
                        self.halt_with_error = true;
                    }

                    else {
                        self.halt_without_error = true;
                    }
                },
            },
            // b"abc" -> binary string
            // b'a' -> binary char
            // f"abc" -> formatted string
            // r"abc" -> raw string
            // br"abc", rb"abc" -> binary raw string
            // fr"abc", rf"abc" -> formatted raw string
            LexState::StringPrefix => match (self.input_buffer.get(self.cursor), self.input_buffer.get(self.cursor + 1), self.input_buffer.get(self.cursor + 2)) {
                // Cannot use the same prefix multiple times.
                (Some(x @ (b'b' | b'f' | b'r')), Some(y @ (b'b' | b'f' | b'r')), Some(z @ (b'"' | b'\''))) if x == y => {
                    self.errors.push(Error {
                        kind: if *z == b'"' {
                            ErrorKind::InvalidStringLiteralPrefix
                        } else {
                            ErrorKind::InvalidCharLiteralPrefix
                        },
                        span: Span::range(
                            self.file,
                            self.cursor + self.offset,
                            self.cursor + 2 + self.offset,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
                (Some(b'b'), Some(b'f'), Some(z @ (b'"' | b'\''))) |
                (Some(b'f'), Some(b'b'), Some(z @ (b'"' | b'\''))) => {
                    self.errors.push(Error {
                        kind: if *z == b'"' {
                            ErrorKind::InvalidStringLiteralPrefix
                        } else {
                            ErrorKind::InvalidCharLiteralPrefix
                        },
                        span: Span::range(
                            self.file,
                            self.cursor + self.offset,
                            self.cursor + 2 + self.offset,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
                (Some(b'b'), Some(b'r'), Some(b'"')) |
                (Some(b'r'), Some(b'b'), Some(b'"')) => {
                    self.state = LexState::StringInit {
                        binary: true,
                        format: false,
                        raw: true,
                    };
                    self.cursor += 2;
                },
                // A binary char is okay, but a raw char is not.
                (Some(x @ b'b'), Some(b'r'), Some(b'\'')) |
                (Some(x @ b'r'), Some(b'b'), Some(b'\'')) => {
                    let error_span = if *x == b'b' {
                        Span::range(
                            self.file,
                            self.cursor + 1 + self.offset,
                            self.cursor + 2 + self.offset,
                        )
                    } else {
                        Span::range(
                            self.file,
                            self.cursor + self.offset,
                            self.cursor + 1 + self.offset,
                        )
                    };
                    self.errors.push(Error {
                        kind: ErrorKind::InvalidCharLiteralPrefix,
                        span: error_span,
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
                (Some(b'f'), Some(b'r'), Some(b'"')) |
                (Some(b'r'), Some(b'f'), Some(b'"')) => {
                    self.state = LexState::StringInit {
                        binary: false,
                        format: true,
                        raw: true,
                    };
                    self.cursor += 2;
                },
                // `f` and `r` are both invalid for a char literal
                (Some(b'f'), Some(b'r'), Some(b'\'')) |
                (Some(b'r'), Some(b'f'), Some(b'\'')) => {
                    self.errors.push(Error {
                        kind: ErrorKind::InvalidCharLiteralPrefix,
                        span: Span::range(
                            self.file,
                            self.cursor + self.offset,
                            self.cursor + 2 + self.offset,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
                (Some(x @ (b'b' | b'f' | b'r')), Some(b'"'), _) => {
                    self.state = LexState::StringInit {
                        binary: *x == b'b',
                        format: *x == b'f',
                        raw: *x == b'r',
                    };
                    self.cursor += 1;
                },
                (Some(b'b'), Some(b'\''), _) => {
                    self.state = LexState::StringInit {
                        binary: true,
                        format: false,
                        raw: false,
                    };
                    self.cursor += 1;
                },
                (Some(b'f' | b'r'), Some(b'\''), _) => {
                    self.errors.push(Error {
                        kind: ErrorKind::InvalidCharLiteralPrefix,
                        span: Span::range(
                            self.file,
                            self.cursor + self.offset,
                            self.cursor + 1 + self.offset,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
                (Some(b'"' | b'\''), _, _) => {
                    self.state = LexState::StringInit {
                        binary: false,
                        format: false,
                        raw: false,
                    };
                },
                (Some(b'a'..=b'z'), Some(b'a'..=b'z'), Some(z @ (b'"' | b'\''))) => {
                    self.errors.push(Error {
                        kind: if *z == b'"' {
                            ErrorKind::InvalidStringLiteralPrefix
                        } else {
                            ErrorKind::InvalidCharLiteralPrefix
                        },
                        span: Span::range(
                            self.file,
                            self.cursor + self.offset,
                            self.cursor + 2 + self.offset,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
                (Some(b'a'..=b'z'), Some(y @ (b'"' | b'\'')), _) => {
                    self.errors.push(Error {
                        kind: if *y == b'"' {
                            ErrorKind::InvalidStringLiteralPrefix
                        } else {
                            ErrorKind::InvalidCharLiteralPrefix
                        },
                        span: Span::range(
                            self.file,
                            self.cursor + self.offset,
                            self.cursor + 1 + self.offset,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
                _ => unreachable!(),
            },
            // `LexState::StringInit` doesn't care even if a char literal has multiple characters.
            // `LexState::Char` will throw an error for that.
            LexState::StringInit { binary, format, raw } => match (
                self.input_buffer.get(self.cursor),
                self.input_buffer.get(self.cursor + 1),
                self.input_buffer.get(self.cursor + 2),
            ) {
                (Some(b'"'), Some(b'"'), Some(b'"')) => {
                    let quote_count = count_quotes(&self.input_buffer, self.cursor).unwrap_or(256);

                    if quote_count % 2 == 0 && quote_count > 254 || quote_count % 2 == 1 && quote_count > 127 {
                        self.errors.push(Error {
                            kind: ErrorKind::TooManyQuotes,
                            span: Span::range(
                                self.file,
                                // I don't want to highlight all the quotes... it's *TooMany*Quotes
                                self.cursor + self.offset,
                                self.cursor + 1 + self.offset,
                            ),
                            ..Error::default()
                        });
                        self.halt_with_error = true;
                        return;
                    }

                    match quote_count {
                        // an empty string literal
                        // for example, if double-quote appears 6 times,
                        // the first 3 starts the literal and the last 3 ends the literal
                        x if x % 4 == 2 => {
                            self.tokens.push(Token {
                                kind: TokenKind::String {
                                    binary,
                                    raw,
                                    s: InternedString::empty(),
                                },
                                span: Span::range(
                                    self.file,
                                    self.cursor + self.offset,
                                    self.cursor + quote_count + self.offset,
                                ),
                            });
                            self.state = LexState::Init;
                            self.cursor += quote_count;
                        },

                        // invalid
                        // a string literal must start with an odd number of quotes
                        x if x % 4 == 0 => {
                            self.errors.push(Error {
                                kind: ErrorKind::WrongNumberOfQuotesInRawStringLiteral,
                                span: Span::range(
                                    self.file,
                                    self.cursor + self.offset,
                                    self.cursor + quote_count + self.offset,
                                ),
                                ..Error::default()
                            });
                            self.halt_with_error = true;
                        },

                        // start of a literal
                        _ => {
                            self.token_start = self.cursor + self.offset;

                            if format {
                                self.state = LexState::FormattedString {
                                    raw,
                                    quote_count,
                                };
                            }

                            else {
                                self.buffer1.clear();
                                self.state = LexState::String {
                                    binary,
                                    raw,
                                    quote_count,
                                };
                            }

                            self.cursor += quote_count;
                        },
                    }
                },
                // an empty string literal
                (Some(b'"'), Some(b'"'), _) => {
                    self.tokens.push(Token {
                        kind: TokenKind::String {
                            binary,
                            raw,
                            s: InternedString::empty(),
                        },
                        span: Span::range(
                            self.file,
                            self.cursor + self.offset,
                            self.cursor + 2 + self.offset,
                        ),
                    });
                    self.state = LexState::Init;
                    self.cursor += 2;
                },
                // an empty char literal -> error!
                (Some(b'\''), Some(b'\''), _) => {
                    self.errors.push(Error {
                        kind: ErrorKind::EmptyCharLiteral,
                        span: Span::range(
                            self.file,
                            self.cursor + self.offset,
                            self.cursor + 2 + self.offset,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
                (Some(b'"'), _, _) => {
                    self.buffer1.clear();
                    self.token_start = self.cursor + self.offset;
                    self.state = LexState::String {
                        binary,
                        raw,
                        quote_count: 1,
                    };
                    self.cursor += 1;
                },
                (Some(b'\''), _, _) => {
                    self.token_start = self.cursor + self.offset;
                    self.state = LexState::Char { binary };
                    self.cursor += 1;
                },
                _ => unreachable!(),
            },
            LexState::String { binary, raw: true, quote_count } => match (
                self.input_buffer.get(self.cursor),
                self.input_buffer.get(self.cursor + 1),
                self.input_buffer.get(self.cursor + 2),
            ) {
                (Some(b'"'), _, _) if quote_count == 1 => {
                    let interned = self.intern_string();

                    self.tokens.push(Token {
                        kind: TokenKind::String {
                            binary,
                            raw: true,
                            s: interned,
                        },
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.cursor + self.offset,
                        ),
                    });
                    self.state = LexState::Init;
                    self.cursor += 1;
                },
                (Some(b'"'), Some(b'"'), Some(b'"')) => {
                    if quote_count == 3 {
                        todo!()
                    }

                    else {
                        let curr_quote_count = count_quotes(&self.input_buffer, self.cursor).unwrap_or(256);

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
                    self.errors.push(Error {
                        kind: ErrorKind::UnterminatedStringLiteral,
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.token_start + quote_count,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
            },
            LexState::String { binary, raw: false, quote_count } => match (
                self.input_buffer.get(self.cursor),
                self.input_buffer.get(self.cursor + 1),
                self.input_buffer.get(self.cursor + 2),
                self.input_buffer.get(self.cursor + 3),
            ) {
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
                (Some(b'\\'), Some(b'x'), Some(z @ (b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z')), Some(w @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F'))) => todo!(),
                // TODO: unicode escape
                // invalid escape
                (Some(b'\\'), Some(y), _, _) => {
                    self.errors.push(Error {
                        kind: ErrorKind::InvalidEscape,
                        span: Span::range(
                            self.file,
                            self.cursor + 1 + self.offset,
                            self.cursor + 2 + self.offset,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
                (Some(b'"'), _, _, _) if quote_count == 1 => {
                    let interned = self.intern_string();
                    self.tokens.push(Token {
                        kind: TokenKind::String {
                            binary,
                            raw: false,
                            s: interned,
                        },
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.cursor + 1 + self.offset,
                        ),
                    });
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
                    self.errors.push(Error {
                        kind: ErrorKind::InvalidUtf8,
                        span: Span::range(
                            self.file,
                            self.cursor + self.offset,
                            self.cursor + 1 + self.offset,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
                (None, _, _, _) => {
                    self.errors.push(Error {
                        kind: ErrorKind::UnterminatedStringLiteral,
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.token_start + quote_count,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
            },
            LexState::FormattedString { raw, quote_count } => todo!(),
            // NOTE: empty char literals are already filtered out!
            // NOTE: the cursor is pointing at the first byte of the content (not the quote)
            LexState::Char { binary } => match (
                self.input_buffer.get(self.cursor),
                self.input_buffer.get(self.cursor + 1),
                self.input_buffer.get(self.cursor + 2),
                self.input_buffer.get(self.cursor + 3),
                self.input_buffer.get(self.cursor + 4),
            ) {
                // valid escape
                (Some(b'\\'), Some(y @ (b'\'' | b'"' | b'\\' | b'n' | b'r' | b't' | b'0')), Some(b'\''), _, _) => {
                    let ch = match *y {
                        b'\'' => '\'',
                        b'"' => '"',
                        b'\\' => '\\',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'0' => '\0',
                        _ => unreachable!(),
                    };
                    self.tokens.push(Token {
                        kind: TokenKind::Char { binary, ch },
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.cursor + 3 + self.offset,
                        ),
                    });
                    self.state = LexState::Init;
                    self.cursor += 3;
                },
                // ascii escape
                (Some(b'\\'), Some(b'x'), Some(z @ (b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z')), Some(w @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')), Some(b'\'')) => todo!(),
                // TODO: unicode escape
                // invalid escape
                (Some(b'\\'), Some(_), _, _, _) => {
                    self.errors.push(Error {
                        kind: ErrorKind::InvalidEscape,
                        span: Span::range(
                            self.file,
                            self.cursor + 1 + self.offset,
                            self.cursor + 2 + self.offset,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
                // invalid char
                (Some(b'\r' | b'\n' | b'\t' | b'\''), _, _, _, _) => {
                    self.errors.push(Error {
                        kind: ErrorKind::InvalidCharLiteral,
                        span: Span::range(
                            self.file,
                            self.cursor + self.offset,
                            self.cursor + 1 + self.offset,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
                // valid char (utf-8)
                (Some(x @ 0..=127), Some(b'\''), _, _, _) => match char::from_u32(*x as u32) {
                    Some(ch) => {
                        self.tokens.push(Token {
                            kind: TokenKind::Char { binary, ch },
                            span: Span::range(
                                self.file,
                                self.token_start,
                                self.cursor + 2 + self.offset,
                            ),
                        });
                        self.state = LexState::Init;
                        self.cursor += 2;
                    },
                    None => {
                        self.errors.push(Error {
                            kind: ErrorKind::InvalidUtf8,
                            span: Span::range(
                                self.file,
                                self.cursor + self.offset,
                                self.cursor + 1 + self.offset,
                            ),
                            ..Error::default()
                        });
                        self.halt_with_error = true;
                    },
                },
                (Some(192..=223), Some(128..=191), Some(b'\''), _, _) => todo!(),
                (Some(224..=239), Some(128..=191), Some(128..=191), Some(b'\''), _) => todo!(),
                (Some(240..=247), Some(128..=191), Some(128..=191), Some(128..=191), Some(b'\'')) => todo!(),
                // invalid utf-8
                (Some(128..), _, _, _, _) => {
                    self.errors.push(Error {
                        kind: ErrorKind::InvalidUtf8,
                        span: Span::range(
                            self.file,
                            self.cursor + self.offset,
                            self.cursor + 1 + self.offset,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
                // etc error (probably multi-character literal)
                (Some(_), _, _, _, _) => {
                    self.errors.push(Error {
                        kind: ErrorKind::InvalidCharLiteral,
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.token_start + 1,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
                (None, _, _, _, _) => {
                    self.errors.push(Error {
                        kind: ErrorKind::UnterminatedCharLiteral,
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.token_start + 1,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
            },
            LexState::Identifier => match self.input_buffer.get(self.cursor) {
                Some(x @ (b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_')) => {
                    self.buffer1.push(*x);
                    self.cursor += 1;
                },
                _ => {
                    let token_kind = match self.buffer1.as_slice() {
                        b"let" => TokenKind::Keyword(Keyword::Let),
                        b"fn" => TokenKind::Keyword(Keyword::Fn),
                        b"struct" => TokenKind::Keyword(Keyword::Struct),
                        b"enum" => TokenKind::Keyword(Keyword::Enum),
                        b"module" => TokenKind::Keyword(Keyword::Module),
                        b"use" => TokenKind::Keyword(Keyword::Use),
                        b"if" => TokenKind::Keyword(Keyword::If),
                        b"else" => TokenKind::Keyword(Keyword::Else),
                        b"pat" => TokenKind::Keyword(Keyword::Pat),
                        b"match" => TokenKind::Keyword(Keyword::Match),
                        _ => {
                            let interned = self.intern_string();
                            TokenKind::Identifier(interned)
                        },
                    };

                    self.tokens.push(Token {
                        kind: token_kind,
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.cursor + self.offset,
                        ),
                    });
                    self.state = LexState::Init;
                },
            },
            LexState::FieldModifier => match self.input_buffer.get(self.cursor) {
                Some(x @ (b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_')) => {
                    self.buffer1.push(*x);
                    self.cursor += 1;
                },
                _ => {
                    let interned = self.intern_string();

                    self.tokens.push(Token {
                        kind: TokenKind::FieldModifier(interned),
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.cursor + self.offset,
                        ),
                    });
                    self.state = LexState::Init;
                },
            },
            LexState::Decorator => match self.input_buffer.get(self.cursor) {
                Some(x @ (b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_')) => {
                    self.buffer1.push(*x);
                    self.cursor += 1;
                },
                _ => {
                    let interned = self.intern_string();

                    self.tokens.push(Token {
                        kind: TokenKind::Decorator(interned),
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.cursor + self.offset,
                        ),
                    });
                    self.state = LexState::Init;
                },
            },
            LexState::Integer(base) => match self.input_buffer.get(self.cursor) {
                Some(x @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')) => {
                    if !base.is_valid_digit(*x) {
                        self.errors.push(Error {
                            kind: ErrorKind::InvalidNumberLiteral,
                            span: Span::range(
                                self.file,
                                self.cursor + self.offset,
                                self.cursor + 1 + self.offset,
                            ),
                            ..Error::default()
                        });
                        self.halt_with_error = true;
                    }

                    self.buffer1.push(*x);
                    self.cursor += 1;
                },
                Some(b'_') => {
                    self.cursor += 1;
                },
                Some(b'.') => match base {
                    Base::Decimal => {
                        self.buffer2.clear();
                        self.state = LexState::Fraction;
                        self.cursor += 1;
                    },
                    Base::Hexadecimal | Base::Octal | Base::Binary => {
                        self.errors.push(Error {
                            kind: ErrorKind::InvalidNumberLiteral,
                            span: Span::range(
                                self.file,
                                self.cursor + self.offset,
                                self.cursor + 1 + self.offset,
                            ),
                            ..Error::default()
                        });
                        self.halt_with_error = true;
                    },
                },
                Some(_) | None => {
                    let interned = intern_number(base, &self.buffer1, &self.buffer2);

                    self.tokens.push(Token {
                        kind: TokenKind::Number(interned),
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.cursor + self.offset,
                        ),
                    });
                    self.state = LexState::Init;
                },
            },
            LexState::Fraction => match self.input_buffer.get(self.cursor) {
                Some(x @ (b'0'..=b'9')) => {
                    self.buffer2.push(*x);
                    self.cursor += 1;
                },
                Some(b'_') => {
                    self.cursor += 1;
                },
                Some(b'e' | b'E') => todo!(),
                Some(_) | None => {
                    // At this point, `Base` must be Decimal. (otherwise lex error)
                    let interned = intern_number(Base::Decimal, &self.buffer1, &self.buffer2);

                    self.tokens.push(Token {
                        kind: TokenKind::Number(interned),
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.cursor + self.offset,
                        ),
                    });
                    self.state = LexState::Init;
                },
            },
            LexState::LineComment => match self.input_buffer.get(self.cursor) {
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
            LexState::DocComment => match self.input_buffer.get(self.cursor) {
                Some(b'\n') => {
                    let interned = self.intern_string();

                    self.tokens.push(Token {
                        kind: TokenKind::DocComment(interned),
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.cursor + self.offset,
                        ),
                    });
                    self.state = LexState::Init;
                    self.cursor += 1;
                },
                Some(x) => {
                    self.buffer1.push(*x);
                    self.cursor += 1;
                },
                None => {
                    self.errors.push(Error {
                        kind: ErrorKind::UnexpectedEof {
                            expected: ErrorToken::Declaration,
                        },
                        span: Span::eof(self.file),
                        ..Error::default()
                    });
                },
            },
            LexState::BlockComment => match (self.input_buffer.get(self.cursor), self.input_buffer.get(self.cursor + 1)) {
                (Some(b'*'), Some(b'/')) => {
                    self.state = LexState::Init;
                    self.cursor += 2;
                },
                (Some(_), _) => {
                    self.cursor += 1;
                },
                (None, _) => {
                    self.errors.push(Error {
                        kind: ErrorKind::UnterminatedBlockComment,

                        // opening of the block comment
                        span: Span::range(
                            self.file,
                            self.token_start,
                            self.token_start + 2,
                        ),
                        ..Error::default()
                    });
                    self.halt_with_error = true;
                },
            },
        }
    }

    fn group_tokens(&mut self) {
        self.tokens = group_tokens_recursive(&self.tokens);
    }

    fn merge_doc_comments(&mut self) {
        let mut new_tokens = vec![];
        let mut doc_comment_buffer = vec![];
        let mut doc_comment_span = Span::None;

        // I can't use `self.tokens.iter` because that will prevent me from calling `self.intern_string`.
        for i in 0..self.tokens.len() {
            let token = self.tokens[i].clone();

            match &token.kind {
                TokenKind::DocComment(line) => {
                    if doc_comment_buffer.is_empty() {
                        doc_comment_span = token.span;
                    } else {
                        doc_comment_span = doc_comment_span.merge(token.span);
                    }

                    doc_comment_buffer.push(*line);
                },
                _ => {
                    if !doc_comment_buffer.is_empty() {
                        // If all the lines are N characters indented, it ignores the N characters indentation.
                        let mut lines = Vec::with_capacity(doc_comment_buffer.len());
                        let mut min_indent = usize::MAX;

                        for line in doc_comment_buffer.iter() {
                            let line = self.unintern_string(*line).unwrap().to_vec();
                            let indent = line.iter().take_while(|b| **b == b' ').count();

                            // If it's an empty line, we should not count its indentation.
                            if indent < line.len() {
                                min_indent = min_indent.min(indent);
                            }

                            lines.push(line);
                        }

                        if min_indent > 0 {
                            lines = lines.iter().map(
                                |line| if line.len() >= min_indent {
                                    line.iter().skip(min_indent).map(|b| *b).collect::<Vec<_>>()
                                } else {
                                    line.to_vec()
                                }
                            ).collect();
                        }

                        self.buffer1 = lines.join(&(b"\n")[..]);
                        new_tokens.push(Token {
                            kind: TokenKind::DocComment(self.intern_string()),
                            span: doc_comment_span,
                        });

                        doc_comment_buffer.clear();
                        doc_comment_span = Span::None;
                    }

                    new_tokens.push(token);
                },
            }
        }

        // lexer guarantees that `Vec<Token>` never ends with `DocComment`.
        assert!(doc_comment_buffer.is_empty());

        self.tokens = new_tokens;
    }

    fn try_load_input(&mut self) -> Result<(), Error> {
        // TODO: If there are more contents to load from the file, it loads more contents to `self.input_buffer` and moves `self.offset`.
        Ok(())
    }

    /// It interns `self.buffer1`. It can't take `self.buffer1` as an input because that would make the borrow checker mad.
    fn intern_string(&mut self) -> InternedString {
        let ins = intern_string(&self.buffer1);

        if !ins.is_short_string() {
            if let Entry::Vacant(e) = self.string_map.entry(ins) {
                e.insert(self.buffer1.to_vec());
            }
        }

        ins
    }

    fn unintern_string(&self, s: InternedString) -> Option<Vec<u8>> {
        if let Some(s) = s.try_unintern_short_string() {
            Some(s)
        }

        else {
            self.string_map.get(&s).map(|s| s.to_vec())
        }
    }
}

// If there's more than 255 quotes, it dies. There's a reason:
// Sodigy-compiler may or may not load the entire file at once.
// So, if it reaches the end of buffer while counting quotes, that
// would be real eof or not-loaded-yet. But it's guaranteed that there's
// at least 255 remaining bytes in the buffer (otherwise eof), so it's
// safe to count quotes up to 255.
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
