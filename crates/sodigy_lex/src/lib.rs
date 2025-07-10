pub struct LexContext {
    buffer: &[u8],
    state: LexState,

    // currently reading `self.buffer[self.cursor]`
    cursor: usize,

    // `self.buffer[i]` is `file.read()[i + self.offset]`
    offset: usize,

    curr_span_start: usize,

    // for parsing numbers
    integer_part: Vec<u8>,
    frac_part: Vec<u8>,
    exp_part: Vec<u8>,

    // for parsing identifiers
    identifier_buffer: Vec<u8>,
    has_multibyte_utf8: bool,
}

impl LexContext {
    pub fn step(&mut self) -> Result<(), LexError> {
        // When it reaches the end of `self.buffer`. The state must be `LexState::Init`.
        // Otherwise, it's a syntax error.
        match self.state {
            LexState::Init => match (self.buffer.get(self.cursor), self.buffer.get(self.cursor + 1)) {
                (Some(b'0'), Some(b'.')) => {
                    self.curr_span_start = self.cursor + self.offset;
                    self.state = LexState::Float;
                    self.cursor += 2;

                    self.integer_part.clear();
                    self.integer_part.push(b'0');
                },
                (Some(b'0'), Some(base @ (b'x' | b'X' | b'o' | b'O' | b'b' | b'B'))) => {
                    self.curr_span_start = self.cursor + self.offset;
                    self.state = LexState::Integer(IntegerParse::from(base));
                    self.cursor += 2;
                },
                (Some(b'0'), Some('e')) => {
                    self.state = LexState::NumberExp;
                    self.cursor += 2;

                    self.integer_part.clear();
                    self.integer_part.push(b'0');
                },
                (Some(b'0'), Some(b'0'..=b'9')) => _,  // err
                (Some(n @ (b'1'..=b'9')), _) => {
                    self.curr_span_start = self.cursor + self.offset;
                    self.state = LexState::Integer(IntegerParse::Decimal);
                    self.cursor += 1;

                    self.integer_part.clear();
                    self.integer_part.push(n);
                },
                (Some(prefix @ (b'r' | b'f' | b'b')), Some(marker @ (b'"' | b'\''))) => {},
                (Some(b'a'..=b'z' | b'A'..=b'Z' | b'_'), _) => {
                    self.curr_span_start = self.cursor + self.offset;
                    self.state = LexState::Identifier;
                    self.cursor += 1;

                    self.has_multibyte_utf8 = false;
                },
                (Some(marker @ (b'"' | b'\'')), _) => {
                    self.curr_span_start = self.cursor + self.offset;
                    self.state = LexState::String;
                    self.cursor += 1;
                },
                (Some(b' ' | b'\n' | b'\r' | b'\t')) => {
                    self.curr_span_start = self.cursor + self.offset;
                    self.state = LexState::Whitespace;
                    self.cursor += 1;
                },
                Some(open @ (b'(' | b'{' | b'[')) => {
                    let delim = Delim::from(open);
                    self.tokens.push(Token::Group {
                        delim,
                        open: true,
                        span: _,
                    });
                    self.groups.push((delim, self.cursor + self.offset));
                    self.cursor += 1;
                },
                Some(close @ (b')' | b'}' | b']')) => match self.groups.pop() {
                    Some((open, start)) if Delim::from(close) == open => {
                        self.tokens.push(Token::Group {
                            delim,
                            open: false,
                            span: _,
                        });
                        self.cursor += 1;
                    },
                    Some((open, start)) => _,  // err
                    None => _,  // err
                },
                (Some(192..), Some(128..)) => {
                    self.curr_span_start = self.cursor + self.offset;
                    self.state = LexState::Identifier;
                    self.cursor += 2;

                    self.has_multibyte_utf8 = true;
                },
                (Some(128..), _) => _,  // err
                (None, _) => {
                    //
                },
            },
            LexState::Integer(integer_parse) => match self.buffer.get(self.cursor) {
                Some(b'_') => {
                    self.cursor += 1;
                },
                Some(n @ (b'0' | b'1')) => {
                    self.cursor += 1;
                    self.integer_part.push(n);
                },
                Some(n @ (b'2'..=b'7')) => match integer_parse {
                    IntegerParse::Binary => _,  // err
                    IntegerParse::Octal
                    | IntegerParse::Decimal
                    | IntegerParse::Hexadecimal => {
                        self.cursor += 1;
                        self.integer_part.push(n);
                    },
                },
                Some(n @ (b'8' | b'9')) => match integer_parse {
                    IntegerParse::Binary
                    | IntegerParse::Octal => _,  // err
                    IntegerParse::Decimal
                    | IntegerParse::Hexadecimal => {
                        self.cursor += 1;
                        self.integer_part.push(n);
                    },
                },
                Some(b'e' | b'E') => match integer_parse {
                    IntegerParse::Binary
                    | IntegerParse::Octal => _,  // err
                    IntegerParse::Decimal => {
                        self.state = LexState::NumberExp;
                        self.cursor += 1;
                    },
                    IntegerParse::Hexadecimal => {
                        self.cursor += 1;
                        self.integer_part.push(b'e');
                    },
                },
                Some(n @ (b'a'..=b'f' | b'A'..=b'F')) => match integer_parse {
                    IntegerParse::Binary
                    | IntegerParse::Octal
                    | IntegerParse::Decimal => _,  // err
                    IntegerParse::Hexadecimal => {
                        self.cursor += 1;
                        self.integer_part.push(n);
                    },
                },
                Some(b'.') => match integer_parse {
                    IntegerParse::Binary
                    | IntegerParse::Octal
                    | IntegerParse::Hexadecimal => _,  // err
                    IntegerParse::Decimal => {
                        self.state = LexState::Float;
                        self.cursor += 1;
                    },
                },
                Some(_) | None => {
                    self.tokens.push(Token::Integer {
                        n: InternedNumber::parse_int(integer_parse, &self.integer_part),
                        span: _,
                    });
                    self.state = LexState::Init;
                },
            },
            LexState::Identifier => match self.buffer.get(self.cursor) {
                Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_') => {
                    self.cursor += 1;
                },
                Some(128..) => {
                    self.cursor += 1;
                    self.has_multibyte_utf8 = true;
                },
                Some(_) | None => {
                    if self.has_multibyte_utf8 {
                        // TODO: check utf-8 validity
                    }

                    self.tokens.push(Token::Identifier {
                        s: intern_string(&self.identifier_buffer),
                        span: _,
                    });
                    self.state = LexState::Init;
                },
            },
        }
    }
}
