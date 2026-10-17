use crate::{Code, ObjectFile};
use sodigy_bytecode::{ExprHash, GlobalLabel, Value};
use sodigy_error::{Error, ErrorKind};
use sodigy_file::File;
use sodigy_number::{BigInt, neg_bi};
use sodigy_token::Delim;
use std::collections::HashMap;

mod lex;
mod token;

use lex::lex;
use token::{
    Command,
    Keyword,
    SectionName,
    Token,
    TokenKind,
};

pub fn parse(bytes: &[u8], file: File, intermediate_dir: &str) -> Result<ObjectFile, Error> {
    let tokens = lex(bytes, file, intermediate_dir)?;
    let mut cursor = 0;

    pop_section_name(SectionName::Data, &tokens, &mut cursor)?;
    let data = parse_data_section(&tokens, &mut cursor)?;

    pop_section_name(SectionName::Code, &tokens, &mut cursor)?;
    let code = parse_code_section(&tokens, &mut cursor)?;
    todo!()
}

fn parse_data_section(tokens: &[Token], cursor: &mut usize) -> Result<HashMap<ExprHash, Value>, Error> {
    let mut result = HashMap::new();

    loop {
        match tokens.get(*cursor) {
            Some(Token { kind: TokenKind::InternedValue(expr_hash), .. }) => {
                *cursor += 1;
                pop_punct(b'=', tokens, cursor)?;
                let value = parse_value(tokens, cursor)?;
                pop_punct(b';', tokens, cursor)?;
                result.insert(ExprHash::from_raw(*expr_hash), value);
            },
            Some(Token { kind: TokenKind::Section(SectionName::Code), .. }) => break,
            Some(b) => {
                return Err(Error {
                    kind: ErrorKind::ObjectFileUnexpectedToken {
                        expected: String::from("%Ixxxx"),
                        got: b.kind.render(),
                    },
                    spans: b.span.simple_error(),
                    note: None,
                });
            },
            None => todo!(),
        }
    }

    Ok(result)
}

fn parse_code_section(tokens: &[Token], cursor: &mut usize) -> Result<HashMap<GlobalLabel, Code>, Error> {
    println!("{:?}", tokens.get(*cursor));
    todo!()
}

fn parse_value(tokens: &[Token], cursor: &mut usize) -> Result<Value, Error> {
    match tokens.get(*cursor) {
        Some(Token { kind: TokenKind::Punct(b'-'), span: neg_span }) => {
            *cursor += 1;

            match tokens.get(*cursor) {
                Some(Token { kind: TokenKind::Scalar(_) | TokenKind::FuncPointer(_), span: n_span }) => todo!(),
                Some(Token { kind: TokenKind::Int(n), .. }) => {
                    *cursor += 1;
                    let (is_neg, nums) = neg_bi(n.is_neg, &n.nums);
                    Ok(Value::Int(BigInt { is_neg, nums }))
                },
                _ => todo!(),
            }
        },
        Some(Token { kind: TokenKind::Scalar(n), .. }) => {
            *cursor += 1;
            Ok(Value::Scalar(*n))
        },
        Some(Token { kind: TokenKind::Int(n), .. }) => {
            *cursor += 1;
            Ok(Value::Int(n.clone()))
        },
        Some(Token { kind: TokenKind::Group { delim, tokens }, .. }) => {
            let mut values = vec![];
            let mut inner_cursor = 0;

            loop {
                if inner_cursor >= tokens.len() {
                    break;
                }

                values.push(parse_value(tokens, &mut inner_cursor)?);

                if inner_cursor >= tokens.len() {
                    break;
                }

                pop_punct(b',', tokens, &mut inner_cursor)?;
            }

            *cursor += 1;

            match delim {
                Delim::Bracket => Ok(Value::List(values)),
                Delim::Brace => Ok(Value::Compound(values)),
                _ => unreachable!(),  // lexer prevents this
            }
        },
        t => panic!("TODO: {t:?}"),
    }
}

fn pop_punct(punct: u8, tokens: &[Token], cursor: &mut usize) -> Result<(), Error> {
    match tokens.get(*cursor) {
        Some(Token { kind: TokenKind::Punct(b), .. }) if *b == punct => {
            *cursor += 1;
            Ok(())
        },
        Some(b) => Err(Error {
            kind: ErrorKind::ObjectFileUnexpectedToken {
                expected: TokenKind::Punct(punct).render(),
                got: b.kind.render(),
            },
            spans: b.span.simple_error(),
            note: None,
        }),
        None => Err(Error {
            kind: ErrorKind::ObjectFileUnexpectedEog {
                expected: Some(TokenKind::Punct(punct).render()),
            },
            spans: vec![],
            note: None,
        }),
    }
}

fn pop_section_name(section: SectionName, tokens: &[Token], cursor: &mut usize) -> Result<(), Error> {
    match tokens.get(*cursor) {
        Some(Token { kind: TokenKind::Section(s), .. }) if *s == section => match tokens.get(*cursor + 1) {
            Some(Token { kind: TokenKind::Punct(b':'), .. }) => {
                *cursor += 2;
                Ok(())
            },
            Some(b) => Err(Error {
                kind: ErrorKind::ObjectFileUnexpectedToken {
                    expected: TokenKind::Section(section).render(),
                    got: b.kind.render(),
                },
                spans: b.span.simple_error(),
                note: None,
            }),
            None => Err(Error {
                kind: ErrorKind::ObjectFileUnexpectedEog {
                    expected: Some(TokenKind::Section(section).render()),
                },
                spans: vec![],
                note: None,
            }),
        },
        Some(b) => Err(Error {
            kind: ErrorKind::ObjectFileUnexpectedToken {
                expected: TokenKind::Section(section).render(),
                got: b.kind.render(),
            },
            spans: b.span.simple_error(),
            note: None,
        }),
        None => Err(Error {
            kind: ErrorKind::ObjectFileUnexpectedEog {
                expected: Some(TokenKind::Section(section).render()),
            },
            spans: vec![],
            note: None,
        }),
    }
}

