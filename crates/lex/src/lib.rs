use sodigy_error::{Error, ErrorKind};
use sodigy_file::File;
use sodigy_keyword::Keyword;
use sodigy_number::{Base, InternedNumber, intern_number};
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};
use std::collections::hash_map::{Entry, HashMap};

mod token;

pub use token::{Token, TokenKind};

pub struct LexSession {
    file: File,
    input_buffer: Vec<u8>,
    state: LexState,
    cursor: usize,
    offset: usize,
    tokens: Vec<Token>,
    string_map: HashMap<InternedString, Vec<u8>>,
    errors: Vec<Error>,

    // cursor + offset of the start of the current token
    token_start: usize,

    // identifier, integer
    buffer1: Vec<u8>,

    // fraction
    buffer2: Vec<u8>,

    // It tries to find as many errors as possible, but with some errors, it cannot continue.
    cannot_continue: bool,
}

enum LexState {
    Init,
    Identifier,
    Integer(Base),
    Fraction,
    LineComment,
    DocComment,
    BlockComment,
}

impl LexSession {
    pub fn step(&mut self) {
        if let Err(e) = self.try_load_input() {
            self.errors.push(Error {
                kind: e.into(),
                span: Span::file(self.file),
            });
            self.cannot_continue = true;
            return;
        }

        if self.cannot_continue {
            return;
        }

        match self.state {
            LexState::Init => match (self.input_buffer.get(self.cursor), self.input_buffer.get(self.cursor + 1), self.input_buffer.get(self.cursor + 2)) {
                (Some(b'f'), Some(b'"' | b'\''), _) => todo!(),
                (Some(b'b'), Some(b'"' | b'\''), _) => todo!(),
                (Some(b'r'), Some(b'"' | b'\''), _) => todo!(),
                (Some(b'"'), Some(b'"'), Some(b'"')) => todo!(),
                (Some(b'"' | b'\''), _, _) => todo!(),
                (Some(x @ (b'a'..=b'z' | b'A'..=b'Z' | b'_')), _, _) => {
                    self.buffer1.clear();
                    self.buffer1.push(*x);

                    self.token_start = self.cursor + self.offset;
                    self.state = LexState::Identifier;
                    self.cursor += 1;
                },
                (Some(b'0'..=b'9'), Some(b'a'..=b'z' | b'A'..=b'Z' | b'_'), _) => {
                    self.errors.push(Error {
                        kind: ErrorKind::InvalidNumberLiteral,
                        span: Span::range(
                            self.file,
                            self.cursor + 1 + self.offset,
                            self.cursor + 2 + self.offset,
                        ),
                    });
                    self.cannot_continue = true;
                },
                (Some(b'0'), Some(b'x' | b'X' | b'o' | b'O' | b'b' | b'B'), _) => todo!(),
                (Some(b'0'), Some(b'.'), _) => todo!(),
                (Some(b'0'), _, _) => {
                    self.tokens.push(Token {
                        kind: TokenKind::Number(todo!()),
                        span: Span::range(
                            self.file,
                            self.cursor + self.offset,
                            self.cursor + 1 + self.offset,
                        ),
                    });
                    self.cursor += 1;
                },
                (Some(b'1'..=b'9'), _, _) => {
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
                (Some(x @ (b'>' | b'<' | b'!' | b'=')), Some(y @ (b'>' | b'<' | b'=')), _) => match (x, y) {
                    (b'>', b'=') => {},
                },
                (Some(b'+' | b'-' | b'*' | b'/' | b'%' | b'>' | b'<' | b'!' | b'='), _, _) => todo!(),
                (Some(_), _, _) => todo!(),
                (None, _, _) => todo!(),
            },
            LexState::Identifier => match self.input_buffer.get(self.cursor) {
                Some(x @ (b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_')) => {
                    self.buffer1.push(*x);
                    self.cursor += 1;
                },
                _ => {
                    let token_kind = match self.buffer1.as_slice() {
                        b"if" => TokenKind::Keyword(Keyword::If),
                        b"let" => TokenKind::Keyword(Keyword::Let),
                        b"match" => TokenKind::Keyword(Keyword::Match),
                        ident => {
                            let interned = self.intern_string(ident);
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
                        });
                        self.cannot_continue = true;
                    }

                    self.buffer1.push(*x);
                    self.cursor += 1;
                },
                Some(b'_') => {
                    self.cursor += 1;
                },
                Some(b'.') => match base {
                    Base::Decimal => {
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
                        });
                        self.cannot_continue = true;
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
                    self.cursor += 1;
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
                    });
                },
            },
        }
    }

    fn intern_string(&mut self, s: &[u8]) -> InternedString {
        let ins = intern_string(s);

        if let Entry::Vacant(e) = self.string_map.entry(ins) {
            e.insert(s.to_vec());
        }

        ins
    }
}
