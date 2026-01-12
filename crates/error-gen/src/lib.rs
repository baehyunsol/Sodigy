extern crate proc_macro;
use proc_macro::{
    Delimiter,
    Group,
    Ident,
    Literal,
    Punct,
    Spacing,
    Span,
    TokenStream,
    TokenTree,
};
use std::collections::HashMap;

enum ParseState {
    ExpectingDef,
    ExpectingComma,
}

#[derive(Clone, Debug)]
struct ErrorKind {
    pub name: String,
    pub fields: Option<Group>,
    pub fields_parsed: EnumFields,
    pub index: u16,
    pub level: String,
}

#[derive(Clone, Debug)]
enum EnumFields {
    None,
    Tuple(Vec<Vec<TokenTree>>),
    Struct(Vec<(String, Vec<TokenTree>)>),
}

enum StructFieldParseState {
    Name,
    Colon,
    Type,
}

impl TryFrom<Group> for EnumFields {
    type Error = TokenStream;

    fn try_from(g: Group) -> Result<EnumFields, TokenStream> {
        match g.delimiter() {
            Delimiter::Parenthesis => {
                let mut fields = vec![];
                let mut curr_field = vec![];

                for token in g.stream() {
                    match token {
                        TokenTree::Punct(p) if p.as_char() == ',' => {
                            fields.push(curr_field);
                            curr_field = vec![];
                        },
                        _ => {
                            curr_field.push(token);
                        },
                    }
                }

                if !curr_field.is_empty() {
                    fields.push(curr_field);
                }

                Ok(EnumFields::Tuple(fields))
            },
            Delimiter::Brace => {
                let mut state = StructFieldParseState::Name;
                let mut fields = vec![];
                let mut curr_id = String::new();
                let mut curr_type = vec![];
                let mut angle_bracket_count = 0;

                for token in g.stream() {
                    match state {
                        StructFieldParseState::Name => match token {
                            TokenTree::Ident(id) => {
                                curr_id = id.to_string();

                                if curr_id.starts_with("r#") {
                                    curr_id = curr_id.get(2..).unwrap().to_string();
                                }

                                state = StructFieldParseState::Colon;
                            },
                            _ => {
                                return Err(error_message(
                                    token.span(),
                                    format!("Expected a name of a field, got `{token:?}`."),
                                ));
                            },
                        },
                        StructFieldParseState::Colon => match token {
                            TokenTree::Punct(p) if p.as_char() == ':' => {
                                state = StructFieldParseState::Type;
                            },
                            _ => {
                                return Err(error_message(
                                    token.span(),
                                    format!("Expected a colon, got `{token:?}`."),
                                ));
                            },
                        },
                        StructFieldParseState::Type => match token {
                            TokenTree::Punct(ref p) if p.as_char() == ',' && angle_bracket_count == 0 => {
                                fields.push((curr_id, curr_type));
                                curr_id = String::new();
                                curr_type = vec![];
                                angle_bracket_count = 0;
                                state = StructFieldParseState::Name;
                            },
                            TokenTree::Punct(ref p) if p.as_char() == '<' => {
                                curr_type.push(token);
                                angle_bracket_count += 1;
                            },
                            TokenTree::Punct(ref p) if p.as_char() == '>' => {
                                curr_type.push(token);
                                angle_bracket_count -= 1;
                            },
                            _ => {
                                curr_type.push(token);
                            },
                        },
                    }
                }

                if !curr_type.is_empty() {
                    fields.push((curr_id, curr_type));
                }

                Ok(EnumFields::Struct(fields))
            },
            d => Err(error_message(
                g.span(),
                format!("Enum fields have to be in parenthesis or braces, but is in `{d:?}`"),
            )),
        }
    }
}

#[proc_macro]
pub fn error_kinds(tokens: TokenStream) -> TokenStream {
    let mut state = ParseState::ExpectingDef;
    let mut definitions = vec![];
    let mut indexes = HashMap::new();
    let mut prev_index = 0;

    for token in tokens {
        match state {
            ParseState::ExpectingDef => {
                match &token {
                    TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis => {
                        let stream = g.stream();

                        if stream.is_empty() {
                            return error_message(
                                g.span(),
                                String::from("Got an empty definition."),
                            );
                        }

                        else {
                            match parse_definition(g.stream()) {
                                Ok(def) => {
                                    if let Some(another_def) = indexes.insert(def.index, def.clone()) {
                                        return error_message(
                                            g.span(),
                                            format!("`{}` and `{}` have the same index ({}).", another_def.name, def.name, def.index),
                                        );
                                    }

                                    if prev_index > def.index {
                                        return error_message(
                                            g.span(),
                                            format!("Please make sure to sort the error kinds by index. {} comes after {}", def.index, prev_index),
                                        );
                                    }

                                    prev_index = def.index;
                                    definitions.push(def);
                                    state = ParseState::ExpectingComma;
                                    continue;
                                },
                                Err(e) => {
                                    return e;
                                },
                            }
                        }
                    },
                    _ => {},
                }

                return error_message(
                    token.span(),
                    format!("Expected a definition, got `{token:?}`."),
                );
            },
            ParseState::ExpectingComma => {
                match token {
                    TokenTree::Punct(p) if p.as_char() == ',' => {
                        state = ParseState::ExpectingDef;
                        continue;
                    },
                    _ => {},
                }

                return error_message(
                    token.span(),
                    format!("Expected a comma, got `{token:?}`."),
                );
            },
        }
    }

    // I'll later use this for a static analysis.
    {
        let dump = definitions.iter().map(
            |def| format!(
                "{}/{}/{}",
                def.name,
                def.index,
                def.level,
            )
        ).collect::<Vec<_>>().join("\n");

        if sodigy_fs_api::exists("crates/error-gen") {
            sodigy_fs_api::write_string(
                "crates/error-gen/errors.txt",
                &dump,
                sodigy_fs_api::WriteMode::CreateOrTruncate,
            ).unwrap();
        }
    }

    render(definitions)
}

fn parse_definition(tokens: TokenStream) -> Result<ErrorKind, TokenStream> {
    let mut last_span = None;
    let mut name = None;
    let mut fields = None;
    let mut fields_parsed = None;
    let mut index = None;
    let mut level = None;
    let mut expecting_comma = false;

    for token in tokens {
        last_span = Some(token.span());

        if expecting_comma {
            match token {
                TokenTree::Punct(p) if p.as_char() == ',' => {
                    expecting_comma = false;
                },
                _ => {
                    return Err(error_message(
                        token.span(),
                        format!("Expected a comma, got `{token:?}`."),
                    ));
                },
            }
        }

        else if name.is_none() {
            match token {
                TokenTree::Ident(id) => {
                    name = Some(id.to_string());
                },
                _ => {
                    return Err(error_message(
                        token.span(),
                        format!("Expected the enum variant of ErrorKind, got `{token:?}`."),
                    ));
                },
            }
        }

        else if fields.is_none() {
            match token {
                TokenTree::Punct(p) if p.as_char() == ',' => {
                    fields = Some(None);
                    fields_parsed = Some(EnumFields::None);
                },
                TokenTree::Group(g) => {
                    fields = Some(Some(g.clone()));
                    fields_parsed = Some(EnumFields::try_from(g)?);
                    expecting_comma = true;
                },
                _ => {
                    return Err(error_message(
                        token.span(),
                        format!("Expected the fields of ErrorKind, got `{token:?}`."),
                    ));
                },
            }
        }

        else if index.is_none() {
            match token {
                TokenTree::Literal(ref lit) => match lit.to_string().parse::<u16>() {
                    Ok(n) => {
                        index = Some(n);
                        expecting_comma = true;

                        if n >= 10000 {
                            return Err(error_message(
                                token.span(),
                                format!("Expected an index in range 0..10000, but got an index {n}."),
                            ));
                        }

                        continue;
                    },
                    _ => {},
                },
                _ => {},
            }

            return Err(error_message(
                token.span(),
                format!("Expected the index of ErrorKind, got `{token:?}`."),
            ));
        }

        else if level.is_none() {
            match token {
                TokenTree::Ident(ref id) => match id.to_string().as_str() {
                    "Error" | "Warning" => {
                        level = Some(id.to_string());
                        expecting_comma = true;
                    },
                    _ => {
                        return Err(error_message(
                            token.span(),
                            format!("{:?} is not a valid level of an error kind. It should be either \"Error\" or \"Warning\".", id.to_string()),
                        ));
                    },
                },
                _ => {
                    return Err(error_message(
                        token.span(),
                        format!("Expected a level of ErrorKind, got `{token:?}`."),
                    ));
                },
            }
        }

        else {
            return Err(error_message(
                token.span(),
                format!("Expected nothing, got `{token:?}`."),
            ));
        }
    }

    match (&name, &fields, &index, &level) {
        (Some(_), Some(_), Some(_), Some(_)) => {},
        _ => {
            let expecting = match (&name, &fields, &index, &level) {
                (None, _, _, _) => "name",
                (_, None, _, _) => "fields",
                (_, _, None, _) => "index",
                (_, _, _, None) => "level",
                _ => unreachable!(),
            };

            return Err(error_message(
                last_span.unwrap(),
                format!("Encountered an unexpected eof while expecting {expecting} of the variant."),
            ));
        },
    }

    Ok(ErrorKind {
        name: name.unwrap(),
        fields: fields.unwrap(),
        fields_parsed: fields_parsed.unwrap(),
        index: index.unwrap(),
        level: level.unwrap(),
    })
}

fn render(definitions: Vec<ErrorKind>) -> TokenStream {
    let mut result = TokenStream::new();
    result.extend(render_enum_definition(&definitions));
    result.extend(render_enum_methods(&definitions));
    result.extend(render_error_level(&definitions));
    result.extend(render_error_kind_endec(&definitions));

    // debug
    {
        let mut dump = vec![];

        for token in result.clone() {
            dump.push(token.to_string());

            match token {
                TokenTree::Punct(p) if p.spacing() == Spacing::Joint => {},
                _ => { dump.push(String::from(" ")); },
            }
        }

        if sodigy_fs_api::exists("crates/error/src/") {
            sodigy_fs_api::write_string(
                "crates/error/src/proc_macro.rs",
                &dump.concat(),
                sodigy_fs_api::WriteMode::CreateOrTruncate,
            ).unwrap();
        }
    }

    result
}

fn render_enum_definition(definitions: &[ErrorKind]) -> TokenStream {
    let variants = definitions.iter().map(
        |def| match &def.fields {
            Some(fields) => vec![
                TokenTree::Ident(Ident::new(&def.name, Span::call_site())),
                TokenTree::Group(fields.clone()),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            ],
            None => vec![
                TokenTree::Ident(Ident::new(&def.name, Span::call_site())),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            ],
        }
    ).collect::<Vec<Vec<TokenTree>>>().concat().into_iter().collect();

    vec![
        TokenTree::Punct(Punct::new('#', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Bracket, vec![
            TokenTree::Ident(Ident::new("derive", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                TokenTree::Ident(Ident::new("Clone", Span::call_site())),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Debug", Span::call_site())),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Eq", Span::call_site())),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Hash", Span::call_site())),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Ident(Ident::new("PartialEq", Span::call_site())),
            ].into_iter().collect())),
        ].into_iter().collect())),
        TokenTree::Ident(Ident::new("pub", Span::call_site())),
        TokenTree::Ident(Ident::new("enum", Span::call_site())),
        TokenTree::Ident(Ident::new("ErrorKind", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Brace, variants)),
    ].into_iter().collect()
}

fn render_enum_methods(definitions: &[ErrorKind]) -> TokenStream {
    let index_match_arms = definitions.iter().map(
        |def| {
            let mut arm = vec![
                TokenTree::Ident(Ident::new("ErrorKind", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new(&def.name, Span::call_site())),
            ];

            match &def.fields_parsed {
                EnumFields::None => {},
                EnumFields::Tuple(t) => {
                    let mut fields = vec![];

                    for _ in t.iter() {
                        fields.push(TokenTree::Ident(Ident::new("_", Span::call_site())));
                        fields.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
                    }

                    arm.push(TokenTree::Group(Group::new(
                        Delimiter::Parenthesis,
                        fields.into_iter().collect(),
                    )));
                },
                EnumFields::Struct(_) => {
                    arm.push(TokenTree::Group(Group::new(
                        Delimiter::Brace,
                        vec![
                            TokenTree::Punct(Punct::new('.', Spacing::Joint)),
                            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                        ].into_iter().collect(),
                    )));
                },
            }

            arm.extend(vec![
                TokenTree::Punct(Punct::new('=', Spacing::Joint)),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                TokenTree::Literal(Literal::u16_suffixed(def.index)),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            ]);
            arm
        }
    ).collect::<Vec<_>>().concat();

    vec![
        TokenTree::Ident(Ident::new("impl", Span::call_site())),
        TokenTree::Ident(Ident::new("ErrorKind", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Brace, vec![
            TokenTree::Ident(Ident::new("pub", Span::call_site())),
            TokenTree::Ident(Ident::new("fn", Span::call_site())),
            TokenTree::Ident(Ident::new("index", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Ident(Ident::new("self", Span::call_site())),
            ].into_iter().collect())),
            TokenTree::Punct(Punct::new('-', Spacing::Joint)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Ident(Ident::new("u16", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Brace, vec![
                TokenTree::Ident(Ident::new("match", Span::call_site())),
                TokenTree::Ident(Ident::new("self", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Brace, index_match_arms.into_iter().collect())),
            ].into_iter().collect())),
        ].into_iter().collect())),
    ].into_iter().collect()
}

fn render_error_level(definitions: &[ErrorKind]) -> TokenStream {
    let arms = definitions.iter().map(
        |def| {
            let mut arm = vec![
                TokenTree::Ident(Ident::new("ErrorKind", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new(&def.name, Span::call_site())),
            ];

            match &def.fields_parsed {
                EnumFields::None => {},
                EnumFields::Tuple(t) => {
                    let mut fields = vec![];

                    for _ in t.iter() {
                        fields.push(TokenTree::Ident(Ident::new("_", Span::call_site())));
                        fields.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
                    }

                    arm.push(TokenTree::Group(Group::new(
                        Delimiter::Parenthesis,
                        fields.into_iter().collect(),
                    )));
                },
                EnumFields::Struct(_) => {
                    arm.push(TokenTree::Group(Group::new(
                        Delimiter::Brace,
                        vec![
                            TokenTree::Punct(Punct::new('.', Spacing::Joint)),
                            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                        ].into_iter().collect(),
                    )));
                },
            }

            arm.extend(vec![
                TokenTree::Punct(Punct::new('=', Spacing::Joint)),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                TokenTree::Ident(Ident::new("ErrorLevel", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new(&def.level, Span::call_site())),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            ]);
            arm
        }
    ).collect::<Vec<_>>().concat();

    vec![
        TokenTree::Ident(Ident::new("impl", Span::call_site())),
        TokenTree::Ident(Ident::new("ErrorLevel", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Brace, vec![
            TokenTree::Ident(Ident::new("pub", Span::call_site())),
            TokenTree::Ident(Ident::new("fn", Span::call_site())),
            TokenTree::Ident(Ident::new("from_error_kind", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                TokenTree::Ident(Ident::new("k", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Ident(Ident::new("ErrorKind", Span::call_site())),
            ].into_iter().collect())),
            TokenTree::Punct(Punct::new('-', Spacing::Joint)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Ident(Ident::new("ErrorLevel", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Brace, vec![
                TokenTree::Ident(Ident::new("match", Span::call_site())),
                TokenTree::Ident(Ident::new("k", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Brace, arms.into_iter().collect())),
            ].into_iter().collect())),
        ].into_iter().collect())),
    ].into_iter().collect()
}

fn render_error_kind_endec(definitions: &[ErrorKind]) -> TokenStream {
    vec![
        TokenTree::Ident(Ident::new("impl", Span::call_site())),
        TokenTree::Ident(Ident::new("Endec", Span::call_site())),
        TokenTree::Ident(Ident::new("for", Span::call_site())),
        TokenTree::Ident(Ident::new("ErrorKind", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Brace, vec![
            render_encode_impl(definitions),
            render_decode_impl(definitions),
        ].concat().into_iter().collect())),
    ].into_iter().collect()
}

fn render_encode_impl(definitions: &[ErrorKind]) -> Vec<TokenTree> {
    let arms: TokenStream = definitions.iter().map(
        |def| {
            let mut body = vec![
                TokenTree::Ident(Ident::new("buffer", Span::call_site())),
                TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                TokenTree::Ident(Ident::new("push", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                    TokenTree::Literal(Literal::u8_suffixed((def.index >> 8) as u8)),
                ].into_iter().collect())),
                TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                TokenTree::Ident(Ident::new("buffer", Span::call_site())),
                TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                TokenTree::Ident(Ident::new("push", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                    TokenTree::Literal(Literal::u8_suffixed((def.index & 0xff) as u8)),
                ].into_iter().collect())),
                TokenTree::Punct(Punct::new(';', Spacing::Alone)),
            ];

            match &def.fields_parsed {
                EnumFields::None => {},
                EnumFields::Tuple(ts) => {
                    for (i, _) in ts.iter().enumerate() {
                        body.push(TokenTree::Ident(Ident::new(&format!("t{i}"), Span::call_site())));
                        body.push(TokenTree::Punct(Punct::new('.', Spacing::Alone)));
                        body.push(TokenTree::Ident(Ident::new("encode_impl", Span::call_site())));
                        body.push(TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                            TokenTree::Ident(Ident::new("buffer", Span::call_site())),
                        ].into_iter().collect())));
                        body.push(TokenTree::Punct(Punct::new(';', Spacing::Alone)));
                    }
                },
                EnumFields::Struct(fs) => {
                    for (field, _) in fs.iter() {
                        body.push(TokenTree::Ident(Ident::new_raw(field, Span::call_site())));
                        body.push(TokenTree::Punct(Punct::new('.', Spacing::Alone)));
                        body.push(TokenTree::Ident(Ident::new("encode_impl", Span::call_site())));
                        body.push(TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                            TokenTree::Ident(Ident::new("buffer", Span::call_site())),
                        ].into_iter().collect())));
                        body.push(TokenTree::Punct(Punct::new(';', Spacing::Alone)));
                    }
                },
            }

            let mut arm = vec![
                TokenTree::Ident(Ident::new("ErrorKind", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new(&def.name, Span::call_site())),
            ];

            match &def.fields_parsed {
                EnumFields::None => {},
                EnumFields::Tuple(ts) => {
                    let mut fields = vec![];

                    for (i, _) in ts.iter().enumerate() {
                        fields.push(TokenTree::Ident(Ident::new(&format!("t{i}"), Span::call_site())));
                        fields.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
                    }

                    arm.push(TokenTree::Group(Group::new(Delimiter::Parenthesis, fields.into_iter().collect())));
                },
                EnumFields::Struct(fs) => {
                    let mut fields = vec![];

                    for (field, _) in fs.iter() {
                        fields.push(TokenTree::Ident(Ident::new_raw(field, Span::call_site())));
                        fields.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
                    }

                    arm.push(TokenTree::Group(Group::new(Delimiter::Brace, fields.into_iter().collect())));
                },
            }

            arm.extend(vec![
                TokenTree::Punct(Punct::new('=', Spacing::Joint)),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                TokenTree::Group(Group::new(Delimiter::Brace, body.into_iter().collect())),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            ]);
            arm
        }
    ).collect::<Vec<_>>().concat().into_iter().collect();

    vec![
        TokenTree::Ident(Ident::new("fn", Span::call_site())),
        TokenTree::Ident(Ident::new("encode_impl", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
            TokenTree::Punct(Punct::new('&', Spacing::Alone)),
            TokenTree::Ident(Ident::new("self", Span::call_site())),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(Ident::new("buffer", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Punct(Punct::new('&', Spacing::Alone)),
            TokenTree::Ident(Ident::new("mut", Span::call_site())),
            TokenTree::Ident(Ident::new("Vec", Span::call_site())),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Ident(Ident::new("u8", Span::call_site())),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
        ].into_iter().collect())),
        TokenTree::Group(Group::new(Delimiter::Brace, vec![
            TokenTree::Ident(Ident::new("match", Span::call_site())),
            TokenTree::Ident(Ident::new("self", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Brace, arms)),
        ].into_iter().collect())),
    ].into_iter().collect()
}

fn render_decode_impl(definitions: &[ErrorKind]) -> Vec<TokenTree> {
    vec![
        TokenTree::Ident(Ident::new("fn", Span::call_site())),
        TokenTree::Ident(Ident::new("decode_impl", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
            TokenTree::Ident(Ident::new("buffer", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Punct(Punct::new('&', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Bracket, vec![
                TokenTree::Ident(Ident::new("u8", Span::call_site())),
            ].into_iter().collect())),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(Ident::new("mut", Span::call_site())),
            TokenTree::Ident(Ident::new("cursor", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("usize", Span::call_site())),
        ].into_iter().collect())),
        TokenTree::Punct(Punct::new('-', Spacing::Joint)),
        TokenTree::Punct(Punct::new('>', Spacing::Alone)),
        TokenTree::Ident(Ident::new("Result", Span::call_site())),
        TokenTree::Punct(Punct::new('<', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
            TokenTree::Ident(Ident::new("Self", Span::call_site())),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(Ident::new("usize", Span::call_site())),
        ].into_iter().collect())),
        TokenTree::Punct(Punct::new(',', Spacing::Alone)),
        TokenTree::Ident(Ident::new("DecodeError", Span::call_site())),
        TokenTree::Punct(Punct::new('>', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Brace, render_decode_impl_body(definitions))),
    ].into_iter().collect()
}

/*
{
    let variant = match (buffer.get(cursor), buffer.get(cursor + 1)) {
        (Some(x), Some(y)) => ((x as u16) << 8) | y as u16,
        _ => {
            return Err(DecodeError::UnexpectedEof);
        },
    };
    cursor += 2;

    match variant {
        0 => Ok((ErrorKind::InvalidNumberLiteral, cursor)),
        5 => {
            let (t0, cursor) = Vec::<u8>::decode_impl(buffer, cursor)?;
            Ok((ErrorKind::InvalidStringLiteralPrefix(t0), cursor))
        },

        // many more variants...

        _ => Err(DecodeError::InvalidLargeEnumVariant(variant)),
    }
}
*/
fn render_decode_impl_body(definitions: &[ErrorKind]) -> TokenStream {
    let mut arms = definitions.iter().map(
        |def| {
            let mut arm = vec![
                TokenTree::Literal(Literal::u16_suffixed(def.index)),
                TokenTree::Punct(Punct::new('=', Spacing::Joint)),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            ];

            match &def.fields_parsed {
                EnumFields::None => {
                    arm.extend(vec![
                        TokenTree::Ident(Ident::new("Ok", Span::call_site())),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                            TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                                TokenTree::Ident(Ident::new("ErrorKind", Span::call_site())),
                                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                                TokenTree::Ident(Ident::new(&def.name, Span::call_site())),
                                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                                TokenTree::Ident(Ident::new("cursor", Span::call_site())),
                            ].into_iter().collect())),
                        ].into_iter().collect())),
                    ]);
                },
                EnumFields::Tuple(ts) => {
                    let mut fields = vec![];
                    let mut body = vec![];

                    for (i, r#type) in ts.iter().enumerate() {
                        let name = format!("t{i}");
                        fields.push(TokenTree::Ident(Ident::new(&name, Span::call_site())));
                        fields.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));

                        body.extend(vec![
                            TokenTree::Ident(Ident::new("let", Span::call_site())),
                            TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                                TokenTree::Ident(Ident::new(&name, Span::call_site())),
                                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                                TokenTree::Ident(Ident::new("cursor", Span::call_site())),
                            ].into_iter().collect())),
                            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                        ]);

                        body.extend(turbo_fish(r#type));

                        body.extend(vec![
                            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                            TokenTree::Ident(Ident::new("decode_impl", Span::call_site())),
                            TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                                TokenTree::Ident(Ident::new("buffer", Span::call_site())),
                                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                                TokenTree::Ident(Ident::new("cursor", Span::call_site())),
                            ].into_iter().collect())),
                            TokenTree::Punct(Punct::new('?', Spacing::Alone)),
                            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                        ]);
                    }

                    body.extend(vec![
                        TokenTree::Ident(Ident::new("Ok", Span::call_site())),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                            TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                                TokenTree::Ident(Ident::new("ErrorKind", Span::call_site())),
                                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                                TokenTree::Ident(Ident::new(&def.name, Span::call_site())),
                                TokenTree::Group(Group::new(Delimiter::Parenthesis, fields.into_iter().collect())),
                                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                                TokenTree::Ident(Ident::new("cursor", Span::call_site())),
                            ].into_iter().collect())),
                        ].into_iter().collect())),
                    ]);
                    arm.push(TokenTree::Group(Group::new(Delimiter::Brace, body.into_iter().collect())));
                },
                EnumFields::Struct(fs) => {
                    let mut fields = vec![];
                    let mut body = vec![];

                    for (name, r#type) in fs.iter() {
                        fields.push(TokenTree::Ident(Ident::new_raw(name, Span::call_site())));
                        fields.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));

                        body.extend(vec![
                            TokenTree::Ident(Ident::new("let", Span::call_site())),
                            TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                                TokenTree::Ident(Ident::new_raw(name, Span::call_site())),
                                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                                TokenTree::Ident(Ident::new("cursor", Span::call_site())),
                            ].into_iter().collect())),
                            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                        ]);

                        body.extend(turbo_fish(r#type));

                        body.extend(vec![
                            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                            TokenTree::Ident(Ident::new("decode_impl", Span::call_site())),
                            TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                                TokenTree::Ident(Ident::new("buffer", Span::call_site())),
                                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                                TokenTree::Ident(Ident::new("cursor", Span::call_site())),
                            ].into_iter().collect())),
                            TokenTree::Punct(Punct::new('?', Spacing::Alone)),
                            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                        ]);
                    }

                    body.extend(vec![
                        TokenTree::Ident(Ident::new("Ok", Span::call_site())),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                            TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                                TokenTree::Ident(Ident::new("ErrorKind", Span::call_site())),
                                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                                TokenTree::Ident(Ident::new(&def.name, Span::call_site())),
                                TokenTree::Group(Group::new(Delimiter::Brace, fields.into_iter().collect())),
                                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                                TokenTree::Ident(Ident::new("cursor", Span::call_site())),
                            ].into_iter().collect())),
                        ].into_iter().collect())),
                    ]);
                    arm.push(TokenTree::Group(Group::new(Delimiter::Brace, body.into_iter().collect())));
                },
            }

            arm.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
            arm
        }
    ).collect::<Vec<_>>().concat();

    arms.extend(vec![
        TokenTree::Ident(Ident::new("_", Span::call_site())),
        TokenTree::Punct(Punct::new('=', Spacing::Joint)),
        TokenTree::Punct(Punct::new('>', Spacing::Alone)),
        TokenTree::Ident(Ident::new("Err", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
            TokenTree::Ident(Ident::new("DecodeError", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("InvalidLargeEnumVariant", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                TokenTree::Ident(Ident::new("variant", Span::call_site())),
                TokenTree::Ident(Ident::new("as", Span::call_site())),
                TokenTree::Ident(Ident::new("u32", Span::call_site())),
            ].into_iter().collect())),
        ].into_iter().collect())),
        TokenTree::Punct(Punct::new(',', Spacing::Alone)),
    ]);

    vec![
        TokenTree::Ident(Ident::new("let", Span::call_site())),
        TokenTree::Ident(Ident::new("variant", Span::call_site())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Ident(Ident::new("match", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
            TokenTree::Ident(Ident::new("buffer", Span::call_site())),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("get", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                TokenTree::Ident(Ident::new("cursor", Span::call_site())),
            ].into_iter().collect())),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(Ident::new("buffer", Span::call_site())),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("get", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                TokenTree::Ident(Ident::new("cursor", Span::call_site())),
                TokenTree::Punct(Punct::new('+', Spacing::Alone)),
                TokenTree::Literal(Literal::usize_suffixed(1)),
            ].into_iter().collect())),
        ].into_iter().collect())),
        TokenTree::Group(Group::new(Delimiter::Brace, vec![
            TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                TokenTree::Ident(Ident::new("Some", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                    TokenTree::Ident(Ident::new("x", Span::call_site())),
                ].into_iter().collect())),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Some", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                    TokenTree::Ident(Ident::new("y", Span::call_site())),
                ].into_iter().collect())),
            ].into_iter().collect())),
            TokenTree::Punct(Punct::new('=', Spacing::Joint)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                    TokenTree::Punct(Punct::new('*', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("x", Span::call_site())),
                    TokenTree::Ident(Ident::new("as", Span::call_site())),
                    TokenTree::Ident(Ident::new("u16", Span::call_site())),
                ].into_iter().collect())),
                TokenTree::Punct(Punct::new('<', Spacing::Joint)),
                TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                TokenTree::Literal(Literal::u32_suffixed(8)),
            ].into_iter().collect())),
            TokenTree::Punct(Punct::new('|', Spacing::Alone)),
            TokenTree::Punct(Punct::new('*', Spacing::Alone)),
            TokenTree::Ident(Ident::new("y", Span::call_site())),
            TokenTree::Ident(Ident::new("as", Span::call_site())),
            TokenTree::Ident(Ident::new("u16", Span::call_site())),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(Ident::new("_", Span::call_site())),
            TokenTree::Punct(Punct::new('=', Spacing::Joint)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Brace, vec![
                TokenTree::Ident(Ident::new("return", Span::call_site())),
                TokenTree::Ident(Ident::new("Err", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, vec![
                    TokenTree::Ident(Ident::new("DecodeError", Span::call_site())),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("UnexpectedEof", Span::call_site())),
                ].into_iter().collect())),
                TokenTree::Punct(Punct::new(';', Spacing::Alone)),
            ].into_iter().collect())),
        ].into_iter().collect())),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        TokenTree::Ident(Ident::new("cursor", Span::call_site())),
        TokenTree::Punct(Punct::new('+', Spacing::Joint)),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Literal(Literal::usize_suffixed(2)),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        TokenTree::Ident(Ident::new("match", Span::call_site())),
        TokenTree::Ident(Ident::new("variant", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Brace, arms.into_iter().collect())),
    ].into_iter().collect()
}

fn turbo_fish(ts: &[TokenTree]) -> Vec<TokenTree> {
    let mut got_angle = false;
    let mut result = vec![];

    for t in ts.iter() {
        match t {
            TokenTree::Punct(p) if p.as_char() == '<' && !got_angle => {
                result.push(TokenTree::Punct(Punct::new(':', Spacing::Joint)));
                result.push(TokenTree::Punct(Punct::new(':', Spacing::Alone)));
                result.push(t.clone());
                got_angle = true;
            },
            _ => {
                result.push(t.clone());
            },
        }
    }

    result
}

fn error_message(span: Span, message: String) -> TokenStream {
    [
        TokenTree::Ident(Ident::new("compile_error", span)),
        TokenTree::Punct(Punct::new('!', Spacing::Alone)),
        TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            [TokenTree::Literal(Literal::string(&message))].into_iter().collect(),
        )),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ].into_iter().collect()
}
