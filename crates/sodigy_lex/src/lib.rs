pub struct LexContext {
    code_buffer: Vec<u8>,
    state: LexState,

    // currently reading `self.code_buffer[self.cursor]`
    cursor: usize,

    // `self.code_buffer[i]` is `file.read()[i + self.offset]`
    offset: usize,

    curr_span_start: usize,

    // current token
    buffer: Vec<u8>,

    // for parsing numbers (integer part is saved to `self.buffer`)
    frac_part: Vec<u8>,
    exp_part: Vec<u8>,

    // for parsing identifiers
    has_multibyte_utf8: bool,

    // for parsing block comments
    block_comment_stack: usize,
}

enum LexState {
    NumberFrac,
    Integer(IntegerLiteralKind),
    NumberExp,
    Identifier,
    String(u8  /* b'"' | b'\'' */),
    Whitespace,
}

impl LexContext {
    pub fn step(&mut self) -> Result<(), LexError> {
        // When it reaches the end of `self.code_buffer`. The state must be `LexState::Init`.
        // Otherwise, it's a syntax error.
        match self.state {
            LexState::Init => match (self.code_buffer.get(self.cursor), self.code_buffer.get(self.cursor + 1)) {
                (Some(b'0'), Some(b'.')) => {
                    self.curr_span_start = self.cursor + self.offset;
                    self.state = LexState::NumberFrac;
                    self.cursor += 2;

                    self.buffer.clear();
                    self.buffer.push(b'0');
                },
                (Some(b'0'), Some(base @ (b'x' | b'X' | b'o' | b'O' | b'b' | b'B'))) => {
                    self.curr_span_start = self.cursor + self.offset;
                    self.state = LexState::Integer(IntegerLiteralKind::from(base));
                    self.cursor += 2;
                },
                (Some(b'0'), Some(b'e')) => {
                    self.state = LexState::NumberExp;
                    self.cursor += 2;

                    self.buffer.clear();
                    self.buffer.push(b'0');
                },
                (Some(b'0'), Some(b'0'..=b'9')) => {
                    return Err(LexError::_);
                },
                (Some(n @ (b'1'..=b'9')), _) => {
                    self.curr_span_start = self.cursor + self.offset;
                    self.state = LexState::Integer(IntegerLiteralKind::Decimal);
                    self.cursor += 1;

                    self.buffer.clear();
                    self.buffer.push(n);
                },
                (Some(prefix @ (b'r' | b'f' | b'b')), Some(marker @ (b'"' | b'\''))) => {
                    self.curr_span_start = self.cursor + self.offset;
                    self.state = LexState::String {
                        prefix: Some(prefix),
                        marker,
                    };
                    self.cursor += 2;
                    self.buffer.clear();
                },
                (Some(marker @ (b'"' | b'\'')), _) => {
                    self.curr_span_start = self.cursor + self.offset;
                    self.state = LexState::String {
                        prefix: None,
                        marker,
                    };
                    self.cursor += 1;
                    self.buffer.clear();
                },
                (Some(b @ (b'a'..=b'z' | b'A'..=b'Z' | b'_')), _) => {
                    self.curr_span_start = self.cursor + self.offset;
                    self.state = LexState::Identifier;
                    self.cursor += 1;

                    self.buffer.clear();
                    self.buffer.push(b);
                    self.has_multibyte_utf8 = false;
                },
                (Some(b' ' | b'\n' | b'\r' | b'\t')) => {
                    self.curr_span_start = self.cursor + self.offset;
                    self.state = LexState::Whitespace;
                    self.cursor += 1;
                },
                Some(open @ (b'(' | b'{' | b'[')) => {
                    let delim = Delim::from(open);
                    self.tokens.push(Token {
                        kind: TokenKind::Group {
                            delim,
                            open: true,
                        },
                        span: _,
                    });
                    self.groups.push((delim, self.cursor + self.offset));
                    self.cursor += 1;
                },
                Some(close @ (b')' | b'}' | b']')) => match self.groups.pop() {
                    Some((open, start)) if Delim::from(close) == open => {
                        self.tokens.push(Token {
                            kind: TokenKind::Group {
                                delim,
                                open: false,
                            },
                            span: _,
                        });
                        self.cursor += 1;
                    },
                    Some((open, start)) => _,  // err
                    None => _,  // err
                },
                (Some(b'/'), Some(b'/')) => {
                    let curr_span_start = self.cursor + self.offset;

                    if let Some(b'/') = self.code_buffer.get(self.cursor + 2) {
                        self.state = LexState::DocComment;
                        self.cursor += 3;
                    }

                    else {
                        self.state = LexState::LineComment;
                        self.cursor += 2;
                    }
                },
                (Some(b'/'), Some(b'*')) => {
                    let curr_span_start = self.cursor + self.offset;
                    self.state = LexState::BlockComment;
                    self.block_comment_stack = 1;
                    self.cursor += 2;
                },
                (Some(a @ 192..), Some(b @ 128..)) => {
                    self.curr_span_start = self.cursor + self.offset;
                    self.state = LexState::Identifier;
                    self.cursor += 2;

                    self.buffer.clear();
                    self.buffer.push(a);
                    self.buffer.push(b);
                    self.has_multibyte_utf8 = true;
                },
                (Some(128..), _) => _,  // err
                // TODO: all multi-character punctuations go here
                // TODO: how about 3-character punctuations?
                (Some(a @ b'='), Some(b @ b'>')) => {
                    self.tokens.push(Token {
                        kind: TokenKind::Punct(intern_string(&[a, b])),
                        span: _,
                    });
                    self.cursor += 2;
                },
                (Some(p), _) => {
                    self.tokens.push(Token {
                        kind: TokenKind::Punct(intern_string(&[p])),
                        span: _,
                    });
                    self.cursor += 1;
                },
                (None, _) => {
                    //
                },
            },
            LexState::Integer(integer_literal_kind) => match self.code_buffer.get(self.cursor) {
                Some(b'_') => {
                    self.cursor += 1;
                },
                Some(n @ (b'0' | b'1')) => {
                    self.cursor += 1;
                    self.buffer.push(n);
                },
                Some(n @ (b'2'..=b'7')) => match integer_literal_kind {
                    IntegerLiteralKind::Binary => _,  // err
                    IntegerLiteralKind::Octal
                    | IntegerLiteralKind::Decimal
                    | IntegerLiteralKind::Hexadecimal => {
                        self.cursor += 1;
                        self.buffer.push(n);
                    },
                },
                Some(n @ (b'8' | b'9')) => match integer_literal_kind {
                    IntegerLiteralKind::Binary
                    | IntegerLiteralKind::Octal => _,  // err
                    IntegerLiteralKind::Decimal
                    | IntegerLiteralKind::Hexadecimal => {
                        self.cursor += 1;
                        self.buffer.push(n);
                    },
                },
                Some(b'e' | b'E') => match integer_literal_kind {
                    IntegerLiteralKind::Binary
                    | IntegerLiteralKind::Octal => _,  // err
                    IntegerLiteralKind::Decimal => {
                        self.state = LexState::NumberExp;
                        self.cursor += 1;
                    },
                    IntegerLiteralKind::Hexadecimal => {
                        self.cursor += 1;
                        self.buffer.push(b'e');
                    },
                },
                Some(n @ (b'a'..=b'f' | b'A'..=b'F')) => match integer_literal_kind {
                    IntegerLiteralKind::Binary
                    | IntegerLiteralKind::Octal
                    | IntegerLiteralKind::Decimal => _,  // err
                    IntegerLiteralKind::Hexadecimal => {
                        self.cursor += 1;
                        self.buffer.push(n);
                    },
                },
                Some(b'.') => match integer_literal_kind {
                    IntegerLiteralKind::Binary
                    | IntegerLiteralKind::Octal
                    | IntegerLiteralKind::Hexadecimal => _,  // err
                    IntegerLiteralKind::Decimal => {
                        self.state = LexState::NumberFrac;
                        self.cursor += 1;
                    },
                },
                Some(_) | None => {
                    let n = match parse_integer(integer_literal_kind, &self.buffer) {
                        Ok(n) => n,
                        Err(_) => _,
                    };

                    self.tokens.push(Token {
                        kind: TokenKind::Integer(intern_number(n)),
                        span: _,
                    });
                    self.state = LexState::Init;
                },
            },
            LexState::Identifier => match self.code_buffer.get(self.cursor) {
                Some(b @ (b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')) => {
                    self.buffer.push(b);
                    self.cursor += 1;
                },
                Some(b @ 128..) => {
                    self.buffer.push(b);
                    self.cursor += 1;
                    self.has_multibyte_utf8 = true;
                },
                Some(_) | None => {
                    if self.has_multibyte_utf8 {
                        if let Err(e) = String::from_utf8(self.buffer) {
                            _  // err
                        }
                    }

                    self.tokens.push(Token {
                        kind: TokenKind::Identifier(intern_string(&self.buffer)),
                        span: _,
                    });
                    self.state = LexState::Init;
                },
            },
            LexState::String { prefix, marker } => match (self.code_buffer.get(self.cursor), self.code_buffer.get(self.cursor + 1)) {
                (Some(b'\\'), Some(b'\\')) | (Some(b'\\'), Some(b'"' | b'\'')) => {
                    self.cursor += 2;
                },
                (Some(b), _) if b == marker => {},
                (Some(b), _) => {},
                (None, _) => {},
            },
            LexState::LineComment => match self.code_buffer.get(self.cursor) {
                // This newline character must become `Token::Whitespace`
                Some(b'\n') | None => {
                    self.tokens.push(Token {
                        kind: TokenKind::LineComment,
                        span: _,
                    });
                    self.state = LexState::Init;
                },
                Some(_) => {
                    self.cursor += 1;
                },
            },
            LexState::DocComment => match self.code_buffer.get(self.cursor) {
                Some(b'\n') | None => {
                    self.tokens.push(Token {
                        kind: TokenKind::DocComment(intern_string(&self.buffer)),
                        span: _,
                    });
                    self.state = LexState::Init;
                },
                Some(b) => {
                    self.buffer.push(b);
                    self.cursor += 1;
                },
            },
            LexState::BlockComment => match (self.code_buffer.get(self.cursor), self.code_buffer.get(self.cursor + 1)) {
                (Some(b'/'), Some(b'*')) => {
                    self.block_comment_stack += 1;
                    self.cursor += 2;
                },
                (Some(b'*'), Some(b'/')) => {
                    self.block_comment_stack -= 1;

                    if self.block_comment_stack == 0 {
                        self.tokens.push(Token {
                            kind: TokenKind::BlockComment,
                            span: _,
                        });
                        self.state = LexState::Init;
                    }

                    self.cursor += 2;
                },
                (Some(_), _) => {
                    self.cursor += 1;
                },
                (None, _) => {
                    _  // err
                },
            },
        }
    }
}
