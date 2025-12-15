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

enum ParseState {
    ExpectingDef,
    ExpectingComma,
}

#[derive(Clone, Debug)]
struct ErrorKind {
    pub name: String,
    pub fields: Option<Group>,
    pub index: u16,
    pub level: String,
}

#[proc_macro]
pub fn error_kinds(tokens: TokenStream) -> TokenStream {
    let mut state = ParseState::ExpectingDef;
    let mut definitions = vec![];

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

    render(definitions)
}

fn parse_definition(tokens: TokenStream) -> Result<ErrorKind, TokenStream> {
    let mut last_span = None;
    let mut name = None;
    let mut fields = None;
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
                    expecting_comma = true;
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
                TokenTree::Ident(id) if id.to_string() == "_" => {
                    fields = Some(None);
                    expecting_comma = true;
                },
                TokenTree::Group(g) => {
                    fields = Some(Some(g.clone()));
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
        index: index.unwrap(),
        level: level.unwrap(),
    })
}

fn render(definitions: Vec<ErrorKind>) -> TokenStream {
    let mut result = TokenStream::new();
    result.extend(render_enum_definition(&definitions));
    result
}

fn render_enum_definition(definitions: &[ErrorKind]) -> TokenStream {
    let variants = definitions.iter().map(
        |def| match &def.fields {
            Some(fields) => vec![
                TokenTree::Ident(Ident::new(&def.name, Span::call_site())),
                TokenTree::Group(fields.clone()),
            ],
            None => vec![TokenTree::Ident(Ident::new(&def.name, Span::call_site()))],
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
            ].into_iter().collect())),
        ].into_iter().collect())),
        TokenTree::Ident(Ident::new("pub", Span::call_site())),
        TokenTree::Ident(Ident::new("enum", Span::call_site())),
        TokenTree::Ident(Ident::new("ErrorKind", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Brace, variants)),
    ].into_iter().collect()
}

fn error_message(span: Span, message: String) -> TokenStream {
    [
        TokenTree::Ident(Ident::new("compile_error", span)),
        TokenTree::Punct(Punct::new('!', Spacing::Alone)),
        TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            [TokenTree::Literal(Literal::string(&message))].into_iter().collect(),
        )),
    ].into_iter().collect()
}
