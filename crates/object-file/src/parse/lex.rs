use super::{Error, Keyword, SectionName, Token};

// TODO: string interning?
pub fn lex(b: &[u8]) -> Result<Vec<Token>, Error> {
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
                let ident = lex_ident(b, &mut cursor)?;
                let token = match ident {
                    b"data" => Token::Section(SectionName::Data),
                    b"code" => Token::Section(SectionName::Code),
                    b"main" => Token::Section(SectionName::Main),
                    b"assert" => Token::Section(SectionName::Assert),
                    _ => todo!(),  // error
                };
                tokens.push(token);
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
                let n = lex_hex_32(b, &mut cursor)?;
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
                let n = lex_hex_32(b, &mut cursor)?;
                tokens.push(Token::LocalLabel(n));
            },
            (Some(b'@'), Some(b'G')) => {
                cursor += 2;
                let n = lex_hex_128(b, &mut cursor)?;
                tokens.push(Token::GlobalLabel(n));
            },
            (Some(b'$'), _) => {
                cursor += 1;
                let command = lex_ident(b, &mut cursor)?;
                tokens.push(Token::Command(command));
            },
            (Some(b'a'..=b'z'), _) => {
                let keyword = lex_ident(b, &mut cursor)?;
                let keyword = match keyword {
                    b"jump" => Keyword::Jump,
                    _ => todo!(),
                };
                tokens.push(Token::Keyword(keyword));
            },
            (None, _) => break,
            (Some(b), _) => todo!(),
        }
    }

    tokens
}

