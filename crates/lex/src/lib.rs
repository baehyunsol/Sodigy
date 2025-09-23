use sodigy_error::{Error, ErrorKind};
use sodigy_file::File;
use sodigy_keyword::Keyword;
use sodigy_number::{Base, InternedNumber, intern_number};
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};
use sodigy_token::{Delim, ErrorToken, Token, TokenKind};
use std::collections::hash_map::{Entry, HashMap};

pub struct LexSession {
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
    Identifier,
    Decorator,
    Integer(Base),
    Fraction,
    LineComment,
    DocComment,
    BlockComment,
}

impl LexSession {
    pub fn gara_init(input: Vec<u8>) -> Self {
        LexSession {
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
        }
    }

    pub fn lex(&mut self) {
        while !self.halt_with_error && !self.halt_without_error {
            self.step();
        }

        if self.errors.is_empty() {
            self.group_tokens();
            self.merge_doc_comments();
        }
    }

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
                    });
                    self.halt_with_error = true;
                },
                (Some(b'0'), Some(b'.'), _) => {
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
                (Some(b'\\'), Some(b'{'), _) => {
                    let opening_span = Span::range(
                        self.file,
                        self.cursor + self.offset,
                        self.cursor + 2 + self.offset,
                    );
                    self.group_stack.push((b'}', opening_span));
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
                        });
                        self.halt_with_error = true;
                    },
                },
                (Some(x @ (b'!' | b'<' | b'=' | b'>')), Some(y @ (b'<' | b'=' | b'>')), _) => match (x, y) {
                    (b'!', b'=') => todo!(),
                    (b'<', b'<') => todo!(),
                    (b'<', b'=') => todo!(),
                    (b'=', b'=') => todo!(),
                    (b'>', b'=') => todo!(),
                    (b'>', b'>') => todo!(),
                    _ => {
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
                (Some(_), _, _) => todo!(),
                (None, _, _) => {
                    if let Some((delim, span)) = self.group_stack.pop() {
                        self.errors.push(Error {
                            kind: ErrorKind::UnclosedDelimiter(delim),
                            span: span,
                        });
                        self.halt_with_error = true;
                    }

                    else {
                        self.halt_without_error = true;
                    }
                },
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
                        b"func" => TokenKind::Keyword(Keyword::Func),
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
                    });
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

        if let Entry::Vacant(e) = self.string_map.entry(ins) {
            e.insert(self.buffer1.to_vec());
        }

        ins
    }

    fn unintern_string(&self, s: InternedString) -> Option<&Vec<u8>> {
        self.string_map.get(&s)
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
