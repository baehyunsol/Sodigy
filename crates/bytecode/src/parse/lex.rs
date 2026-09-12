use super::{BytecodeParseError, Section, Token};
use crate::Value;
use sodigy_number::{BigInt, or_ubi, shl_ubi};
use sodigy_span::{Span, SpanHash};

pub fn lex(b: &[u8]) -> Result<[Vec<Token>; 3], BytecodeParseError> {
    let mut cursor = 0;
    let mut data = None;
    let mut code = None;
    let mut label = None;

    loop {
        match (b.get(cursor), b.get(cursor + 1)) {
            (Some(b'.'), _) => {
                if b.len() > cursor + 6 && &b[cursor..(cursor + 6)] == b".data:" {
                    if data.is_some() {
                        return Err(BytecodeParseError::DuplicateSection(Section::Data));
                    }

                    cursor += 6;
                    let (new_data, new_cursor) = lex_data_section(b, cursor)?;
                    data = Some(new_data);
                    cursor = new_cursor;
                } else if b.len() > cursor + 6 && &b[cursor..(cursor + 6)] == b".code:" {
                    if code.is_some() {
                        return Err(BytecodeParseError::DuplicateSection(Section::Code));
                    }

                    cursor += 6;
                    let (new_code, new_cursor) = lex_code_section(b, cursor)?;
                    code = Some(new_code);
                    cursor = new_cursor;
                } else if b.len() > cursor + 7 && &b[cursor..(cursor + 7)] == b".label:" {
                    if label.is_some() {
                        return Err(BytecodeParseError::DuplicateSection(Section::Label));
                    }

                    cursor += 7;
                    let (new_label, new_cursor) = lex_label_section(b, cursor)?;
                    label = Some(new_label);
                    cursor = new_cursor;
                } else {
                    return Err(BytecodeParseError::InvalidSection { cursor });
                }
            },
            (Some(b' ' | b'\n' | b'\t' | b'\r'), _) => {
                cursor += 1;
            },
            (Some(b'/'), Some(b'/')) => {
                cursor += 2;

                while let Some(b) = b.get(cursor) && *b != b'\n' {
                    cursor += 1;
                }
            },
            (Some(b), _) => {
                return Err(BytecodeParseError::UnexpectedByte {
                    expected: Some(b'.'),
                    got: *b,
                    cursor,
                });
            },
            (None, _) => match (data, code, label) {
                (Some(data), Some(code), Some(label)) => {
                    return Ok([data, code, label]);
                },
                (None, _, _) => {
                    return Err(BytecodeParseError::MissingSection(Section::Data));
                },
                (_, None, _) => {
                    return Err(BytecodeParseError::MissingSection(Section::Code));
                },
                (_, _, None) => {
                    return Err(BytecodeParseError::MissingSection(Section::Label));
                },
            },
        }
    }
}

fn lex_data_section(b: &[u8], mut cursor: usize) -> Result<(Vec<Token>, usize), BytecodeParseError> {
    let mut tokens = vec![];

    loop {
        match (b.get(cursor), b.get(cursor + 1)) {
            (Some(b' ' | b'\n' | b'\t' | b'\r'), _) => {
                cursor += 1;
            },
            (Some(b'/'), Some(b'/')) => {
                cursor += 2;

                while let Some(b) = b.get(cursor) && *b != b'\n' {
                    cursor += 1;
                }
            },
            (Some(b'%'), Some(b'I')) => {
                cursor += 2;
                let int_start_cursor = cursor;
                let (value, new_cursor) = lex_hex(b, cursor)?;
                cursor = new_cursor;

                match u128::try_from(&value) {
                    Ok(value) => {
                        tokens.push(Token::InternedValue(value));
                    },
                    Err(_) => {
                        return Err(BytecodeParseError::IntRangeError { cursor: int_start_cursor });
                    },
                }
            },
            (Some(b'='), _) => {
                cursor += 1;
                tokens.push(Token::Assign);
            },
            (Some(b';'), _) => {
                cursor += 1;
                tokens.push(Token::Semicolon);
            },
            (Some(b'.') | None, _) => {
                return Ok((tokens, cursor));
            },
            (Some(_), _) => {
                let (value, new_cursor) = lex_value(b, cursor)?;
                cursor = new_cursor;
                tokens.push(Token::Value(value));
            },
        }
    }
}

fn lex_code_section(b: &[u8], mut cursor: usize) -> Result<(Vec<Token>, usize), BytecodeParseError> {
    let mut tokens = vec![];

    loop {
        match (b.get(cursor), b.get(cursor + 1)) {
            (Some(b' ' | b'\n' | b'\t' | b'\r'), _) => {
                cursor += 1;
            },
            (Some(b'/'), Some(b'/')) => {
                cursor += 2;

                while let Some(b) = b.get(cursor) && *b != b'\n' {
                    cursor += 1;
                }
            },
            (Some(b'.') | None, _) => {
                return Ok((tokens, cursor));
            },
            (Some(b'#'), Some(b'[')) => {
                cursor += 2;

                let (ident, new_cursor) = lex_ident(b, cursor)?;
                cursor = new_cursor;
                let (args, new_cursor) = lex_idents(b, cursor)?;
                cursor = new_cursor;

                match b.get(cursor) {
                    Some(b']') => {
                        cursor += 1;
                    },
                    Some(b) => {
                        return Err(BytecodeParseError::UnexpectedByte {
                            expected: Some(b']'),
                            got: *b,
                            cursor,
                        });
                    },
                    None => {
                        return Err(BytecodeParseError::UnexpectedEnd);
                    },
                }

                tokens.push(Token::Decorator {
                    ident,
                    args,
                });
            },
            _ => todo!(),
        }
    }
}

fn lex_label_section(b: &[u8], mut cursor: usize) -> Result<(Vec<Token>, usize), BytecodeParseError> {
    let mut tokens = vec![];

    loop {
        match (b.get(cursor), b.get(cursor + 1)) {
            (Some(b' ' | b'\n' | b'\t' | b'\r'), _) => {
                cursor += 1;
            },
            (Some(b'/'), Some(b'/')) => {
                cursor += 2;

                while let Some(b) = b.get(cursor) && *b != b'\n' {
                    cursor += 1;
                }
            },
            (Some(b'.') | None, _) => {
                return Ok((tokens, cursor));
            },
            _ => todo!(),
        }
    }
}

fn lex_hex(b: &[u8], mut cursor: usize) -> Result<(BigInt, usize), BytecodeParseError> {
    let mut nums = vec![0];
    let mut is_neg = false;

    match b.get(cursor) {
        Some(b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F') => {},
        Some(b'-') => {
            is_neg = true;
            cursor += 1;
        },
        _ => {
            return Err(BytecodeParseError::FailedToParseHex { cursor });
        },
    }

    loop {
        match b.get(cursor) {
            Some(b @ b'0'..=b'9') => {
                nums = shl_ubi(&nums, 4);
                nums = or_ubi(&nums, &[(*b - b'0') as u32]);
                cursor += 1;
            },
            Some(b @ b'a'..=b'f') => {
                nums = shl_ubi(&nums, 4);
                nums = or_ubi(&nums, &[(*b - b'a' + 10) as u32]);
                cursor += 1;
            },
            Some(b @ b'A'..=b'F') => {
                nums = shl_ubi(&nums, 4);
                nums = or_ubi(&nums, &[(*b - b'A' + 10) as u32]);
                cursor += 1;
            },
            _ => {
                return Ok((
                    BigInt { is_neg, nums },
                    cursor,
                ));
            },
        }
    }
}

fn lex_value(b: &[u8], mut cursor: usize) -> Result<(Value, usize), BytecodeParseError> {
    match b.get(cursor) {
        Some(b'#') => match b.get(cursor + 1) {
            // currently, this format cannot encode/decode spans
            Some(b'p') => Ok((Value::Span(Span::None), cursor + 2)),
            _ => Err(BytecodeParseError::FailedToParseValue { cursor: cursor + 1 }),
        },
        Some(start @ (b'[' | b'{')) => {
            let end = match *start { b'[' => b']', b'{' => b'}', _ => unreachable!() };
            let mut expecting_value = true;
            let mut values = vec![];
            cursor += 1;

            loop {
                if expecting_value {
                    match (b.get(cursor), b.get(cursor + 1)) {
                        (Some(b' ' | b'\n' | b'\t' | b'\r'), _) => {
                            cursor += 1;
                        },
                        (Some(b'/'), Some(b'/')) => {
                            cursor += 2;

                            while let Some(b) = b.get(cursor) && *b != b'\n' {
                                cursor += 1;
                            }
                        },
                        (Some(b), _) if *b == end => {
                            cursor += 1;
                            break;
                        },
                        (Some(_), _) => {
                            let (value, new_cursor) = lex_value(b, cursor)?;
                            values.push(value);
                            cursor = new_cursor;
                            expecting_value = false;
                        },
                        (None, _) => {
                            return Err(BytecodeParseError::FailedToParseValue { cursor });
                        },
                    }
                }

                // expecting comma
                else {
                    match (b.get(cursor), b.get(cursor + 1)) {
                        (Some(b' ' | b'\n' | b'\t' | b'\r'), _) => {
                            cursor += 1;
                        },
                        (Some(b'/'), Some(b'/')) => {
                            cursor += 2;

                            while let Some(b) = b.get(cursor) && *b != b'\n' {
                                cursor += 1;
                            }
                        },
                        (Some(b','), _) => {
                            cursor += 1;
                            expecting_value = true;
                        },
                        (Some(b), _) if *b == end => {
                            cursor += 1;
                            break;
                        },
                        (Some(_) | None, _) => {
                            return Err(BytecodeParseError::FailedToParseValue { cursor });
                        },
                    }
                }
            }

            if *start == b'[' {
                Ok((Value::List(values), cursor))
            } else {
                Ok((Value::Compound(values), cursor))
            }
        },
        Some(_) => {
            let int_start_cursor = cursor;
            let (value, new_cursor) = lex_hex(b, cursor)?;
            cursor = new_cursor;

            match (b.get(cursor), b.get(cursor + 1)) {
                (Some(b'#'), Some(b'f')) => match u128::try_from(&value) {
                    Ok(s) => Ok((
                        Value::FuncPointer {
                            def_span: SpanHash(s),
                            program_counter: None,
                        },
                        cursor + 2,
                    )),
                    Err(_) => Err(BytecodeParseError::IntRangeError { cursor: int_start_cursor }),
                },
                (Some(b'#'), Some(b'n')) => Ok((Value::Int(value), cursor + 2)),
                (Some(b'#'), Some(b's')) => match u32::try_from(&value) {
                    Ok(n) => Ok((Value::Scalar(n), cursor + 2)),
                    Err(_) => Err(BytecodeParseError::IntRangeError {
                        cursor: int_start_cursor,
                    }),
                },
                _ => Err(BytecodeParseError::FailedToParseValue { cursor }),
            }
        },
        None => Err(BytecodeParseError::FailedToParseValue { cursor }),
    }
}

fn lex_ident(b: &[u8], mut cursor: usize) -> Result<(Vec<u8>, usize), BytecodeParseError> {
    let mut buffer = vec![];

    match b.get(cursor) {
        Some(b @ (b'a'..=b'z' | b'A'..=b'Z' | b'_')) => {
            buffer.push(*b);
            cursor += 1;
        },
        _ => {
            return Err(BytecodeParseError::FailedToParseIdent { cursor });
        },
    }

    loop {
        match b.get(cursor) {
            Some(b @ (b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')) => {
                buffer.push(*b);
                cursor += 1;
            },
            _ => {
                return Ok((buffer, cursor));
            },
        }
    }
}

fn lex_idents(b: &[u8], mut cursor: usize) -> Result<(Vec<Vec<u8>>, usize), BytecodeParseError> {
    todo!()
}
