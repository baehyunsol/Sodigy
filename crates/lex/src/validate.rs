use crate::Session;
use sodigy_string::unintern_string;
use sodigy_token::{Token, TokenKind, TokensOrString};

impl Session {
    // It panics if it fails to validate!
    pub fn validate_spans(&self) {
        validate_spans_worker(&self.input_bytes, &self.tokens, &self.intermediate_dir);
    }
}

fn validate_spans_worker(
    file_content: &[u8],
    tokens: &[Token],
    intermediate_dir: &str,
) {
    for Token { kind, span } in tokens.iter() {
        let (start, end) = span.get_bounds().unwrap();
        let span_code = &file_content[(start as usize)..(end as usize)];

        match kind {
            TokenKind::Keyword(k) => {
                let k = format!("{k:?}").to_ascii_lowercase();
                assert_eq!(k.as_bytes(), span_code);
            },
            TokenKind::Ident(i) => {
                let i = unintern_string(*i, intermediate_dir).unwrap().unwrap();
                let raw_i = [b"r#".to_vec(), i.to_vec()].concat();

                if i != span_code && raw_i != span_code {
                    panic!("span_code: {span_code:?}, identifier: {i:?}");
                }
            },
            TokenKind::Number(_) => {
                let mut has_dot = false;

                if let b'0'..=b'9' = span_code[0] {
                    //
                } else {
                    panic!("{span_code:?}");
                }

                for c in span_code.iter() {
                    match c {
                        b'0'..=b'9' => {},
                        b'.' => {
                            assert!(!has_dot);
                            has_dot = true;
                        },
                        b'-' | b'_' | b'x' | b'X' | b'o' | b'O' | b'b' | b'B' | b'e' => {},
                        _ => panic!("{span_code:?}"),
                    }
                }
            },
            TokenKind::String { .. } => {
                assert!(matches!(span_code[0], b'r' | b'f' | b'b' | b'"'));
                assert_eq!(span_code[span_code.len() - 1], b'"');
            },
            TokenKind::Char(_) => {
                assert_eq!(span_code[0], b'\'');
                assert_eq!(span_code[span_code.len() - 1], b'\'');
            },
            TokenKind::Byte(_) => {
                match span_code[0] {
                    b'#' => match span_code[1] {
                        b'0'..=b'9' => {},
                        _ => panic!("{span_code:?}"),
                    },
                    b'b' => match span_code[1] {
                        b'\'' => {},
                        _ => panic!("{span_code:?}"),
                    },
                    _ => panic!("{span_code:?}"),
                }
            },
            TokenKind::FormattedString { raw, elements } => {
                if *raw {
                    if !span_code.starts_with(b"rf\"") && !span_code.starts_with(b"fr\"") {
                        panic!("{span_code:?}");
                    }
                } else {
                    assert_eq!(&span_code[0..2], b"f\"");
                }

                for element in elements.iter() {
                    match element {
                        TokensOrString::Tokens { tokens, span } => {
                            let (start, end) = span.get_bounds().unwrap();
                            assert_eq!(file_content[start as usize], b'{');
                            assert_eq!(file_content[end as usize], b'}');

                            validate_spans_worker(file_content, tokens, intermediate_dir);
                        },
                        TokensOrString::String { s, span } => {
                            let (start, end) = span.get_bounds().unwrap();
                            let span_code = file_content[(start as usize)..(end as usize)].to_vec();
                            let mut span_code_processed = Vec::with_capacity(span_code.len());
                            let mut i = 0;

                            loop {
                                match (span_code.get(i), span_code.get(i + 1)) {
                                    (Some(b'{'), Some(b'{')) => {
                                        span_code_processed.push(b'{');
                                        i += 2;
                                    },
                                    (Some(b'{'), _) => unreachable!(),
                                    (Some(b'}'), Some(b'}')) => {
                                        span_code_processed.push(b'}');
                                        i += 2;
                                    },
                                    (Some(b'}'), _) => unreachable!(),
                                    (Some(c), _) => {
                                        span_code_processed.push(*c);
                                        i += 1;
                                    },
                                    _ => {
                                        break;
                                    },
                                }
                            }

                            assert_eq!(
                                unintern_string(*s, intermediate_dir).unwrap().unwrap(),
                                span_code_processed,
                            );
                        },
                    }
                }
            },
            TokenKind::FieldUpdate { field, backtick_span, field_span } => {
                let (start, end) = backtick_span.get_bounds().unwrap();
                assert_eq!(&file_content[(start as usize)..(end as usize)], b"`");

                let (start, end) = field_span.get_bounds().unwrap();
                assert_eq!(
                    unintern_string(*field, intermediate_dir).unwrap().unwrap(),
                    &file_content[(start as usize)..(end as usize)],
                );
            },
            TokenKind::DocComment { top_level, .. } => {
                if *top_level {
                    assert_eq!(&span_code[0..3], b"//!");
                } else {
                    assert_eq!(&span_code[0..3], b"///");
                }

                match file_content.get(end as usize) {
                    None | Some(b'\n') => {},
                    _ => panic!("{span_code:?}"),
                }
            },
            TokenKind::Punct(p) => {
                assert_eq!(span_code, p.render_error().as_bytes());
            },
            TokenKind::GroupDelim { .. } => unreachable!(),
            TokenKind::Group { delim, tokens } => {
                let (open, close) = delim.markers();
                assert!(span_code.starts_with(open.as_bytes()));
                assert!(span_code.ends_with(close.as_bytes()));
                validate_spans_worker(file_content, tokens, intermediate_dir);
            },
            TokenKind::Wildcard => {
                assert_eq!(span_code, b"_");
            },
        }
    }
}
