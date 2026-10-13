use super::{Keyword, SectionName, Token, TokenKind};
use sodigy_error::{Error, ErrorKind};
use sodigy_file::File;
use sodigy_span::Span;

// TODO: string interning?
pub fn lex(b: &[u8], file: File) -> Result<Vec<Token>, Error> {
    let mut cursor = 0;
    let mut tokens = vec![];

    loop {
        match (b.get(cursor), b.get(cursor + 1)) {
            (Some(b' ' | b'\t' | b'\r' | b'\n'), _) => {
                cursor += 1;
            },
            (Some(b'/'), Some(b'/')) => {
                cursor += 1;

                while let Some(b) = b.get(cursor) && *b != b'\n' {
                    cursor += 1;
                }
            },
            (Some(b'"'), _) => todo!(),
            (Some(b'(' | b'[' | b'{'), _) => todo!(),
            (Some(b'.'), _) => {
                cursor += 1;
                let old_cursor = cursor;
                let ident = lex_ident(b, &mut cursor)?;
                let token_kind = match &b[old_cursor..cursor] {
                    b"data" => TokenKind::Section(SectionName::Data),
                    b"code" => TokenKind::Section(SectionName::Code),
                    b"main" => TokenKind::Section(SectionName::Main),
                    b"assert" => TokenKind::Section(SectionName::Assert),
                    b => {
                        return Err(Error {
                            kind: ErrorKind::InvalidSectionName(b),
                            spans: Span::range(file, old_cursor as u32, b.len() as u32).simple_error(),
                            note: None,
                        });
                    },
                };
                tokens.push(Token {
                    kind: token_kind,
                    span: Span::range(file, old_cursor as u32, (cursor - old_cursor) as u32),
                });
            },
            (Some(b'_'), Some(b'x')) => {
                cursor += 2;
                let n = lex_dec_32(b, &mut cursor)?;
                tokens.push(Token::SSA(n));
            },
            (Some(b'_'), Some(b'g')) => {
                cursor += 2;
                let n = lex_hex_128(b, &mut cursor)?;
                tokens.push(Token::GlobalValue(n));
            },
            (Some(b'%'), Some(b'I')) => {
                cursor += 2;
                let n = lex_hex_128(b, &mut cursor)?;
                tokens.push(Token::InternedValue(n));
            },
            (Some(b'#'), Some(b's')) => {
                cursor += 2;
                let n = lex_hex_32(b, &mut cursor, file)?;
                tokens.push(Token::Scalar(n));
            },
            (Some(b'#'), Some(b'f')) => {
                cursor += 2;
                let n = lex_hex_128(b, &mut cursor)?;
                tokens.push(Token::FuncPointer(n));
            },
            (Some(b'#'), Some(b'n')) => {
                cursor += 2;
                let n = lex_hex_large(b, &mut cursor)?;
                tokens.push(Token::Integer(n));
            },
            (Some(b'@'), Some(b'L')) => {
                cursor += 2;
                let n = lex_hex_32(b, &mut cursor, file)?;
                tokens.push(Token::LocalLabel(n));
            },
            (Some(b'@'), Some(b'G')) => {
                let old_cursor = cursor;
                cursor += 2;
                let n = lex_hex_128(b, &mut cursor)?;
                tokens.push(Token {
                    kind: TokenKind::GlobalLabel(n),
                    span: Span::range(file, old_cursor as u32, (cursor - old_cursor) as u32),
                });
            },
            (Some(b'$'), _) => {
                let old_cursor = cursor;
                cursor += 1;
                let command = lex_ident(b, &mut cursor, file)?;
                let command = Command::from_ident(&b[(old_cursor + 1)..cursor])?;
                tokens.push(Token {
                    kind: TokenKind::Command(command),
                    span: Span::range(file, old_cursor as u32, (cursor - old_cursor) as u32),
                });
            },
            (Some(b'a'..=b'z'), _) => {
                let old_cursor = cursor;
                let keyword = lex_ident(b, &mut cursor, file)?;
                let token_kind = match &b[old_cursor..cursor] {
                    b"jump" => TokenKind::Keyword(Keyword::Jump),
                    _ => todo!(),
                };
                tokens.push(Token {
                    kind: token_kind,
                    span: Span::range(file, old_cursor as u32, (cursor - old_cursor) as u32),
                });
            },
            (None, _) => break,
            (Some(b), _) => {
                return Err(Error {
                    kind: ErrorKind::UnexpectedByte(*b),
                    spans: Span::range(file, cursor as u32, 1).simple_error(),
                    note: None,
                });
            },
        }
    }

    Ok(tokens)
}

// If there's a valid ident, it just moves the cursor and
// returns `Ok(())`.
fn lex_ident(b: &[u8], cursor: &mut usize, file: File) -> Result<(), Error> {
    let old_cursor = *cursor;

    match b.get(*cursor) {
        Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => {
            *cursor += 1;
        },
        Some(b) => return Err(Error {
            kind: ErrorKind::InvalidCharacterInIdent(*b as char),
            spans: Span::range(file, old_cursor as u32, 1).simple_error(),
            note: None,
        }),
        None => return Err(Error {
            kind: ErrorKind::EmptyIdent,
            spans: Span::range(file, old_cursor as u32, 1).simple_error(),
            note: None,
        }),
    }

    loop {
        match b.get(*cursor) {
            Some(b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_') => {
                *cursor += 1;
            },
            _ => {
                return Ok(());
            },
        }
    }
}

fn lex_hex_32(b: &[u8], cursor: &mut usize, file: File) -> Result<u32, Error> {
    let old_cursor = *cursor;
    let mut buffer: u64 = match b.get(*cursor) {
        Some(b @ b'0'..=b'9') => (*b - b'0') as u64,
        Some(b @ b'a'..=b'f') => (*b - b'a' + 10) as u64,
        Some(b @ b'A'..=b'F') => (*b - b'A' + 10) as u64,
        Some(b) => todo!(),
        None => todo!(),
    };
    *cursor += 1;

    loop {
        match b.get(*cursor) {
            Some(b @ b'0'..=b'9') => {
                buffer <<= 4;
                buffer |= (*b - b'0') as u64;
            },
            Some(b @ b'a'..=b'f') => {
                buffer <<= 4;
                buffer |= (*b - b'a' + 10) as u64;
            },
            Some(b @ b'A'..=b'F') => {
                buffer <<= 4;
                buffer |= (*b - b'A' + 10) as u64;
            },
            _ => {
                return Ok(buffer as u32);
            },
        }

        if buffer > u32::MAX as u64 {
            todo!()  // error
        }

        *cursor += 1;
    }
}

