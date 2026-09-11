use super::{BytecodeParseError, Section, Token};
use crate::Value;

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
                let (value, new_cursor) = lex_hex(b, cursor)?;
                cursor = new_cursor;
                tokens.push(Token::InternedValue(value));
            },
            (Some(b'='), _) => {
                tokens.push(Token::Assign);
            },
            (Some(b';'), _) => {
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

fn lex_hex(b: &[u8], mut cursor: usize) -> Result<(u128, usize), BytecodeParseError> {
    let mut buffer = 0;

    match b.get(cursor) {
        Some(b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F') => {},
        _ => {
            return Err(BytecodeParseError::FailedToParseHex { cursor });
        },
    }

    loop {
        match b.get(cursor) {
            Some(b @ b'0'..=b'9') => {
                buffer <<= 4;
                buffer |= (*b - b'0') as u128;
                cursor += 1;
            },
            Some(b @ b'a'..=b'f') => {
                buffer <<= 4;
                buffer |= (*b - b'a' + 10) as u128;
                cursor += 1;
            },
            Some(b @ b'A'..=b'F') => {
                buffer <<= 4;
                buffer |= (*b - b'A' + 10) as u128;
                cursor += 1;
            },
            _ => {
                return Ok((buffer, cursor));
            },
        }
    }
}

fn lex_value(b: &[u8], mut cursor: usize) -> Result<(Value, usize), BytecodeParseError> {
    todo!()
}
