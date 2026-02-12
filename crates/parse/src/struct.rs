use crate::{
    Attribute,
    Expr,
    FuncParam,
    Generic,
    Tokens,
};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Delim, Keyword, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Struct {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<Generic>,
    pub generic_group_span: Option<Span>,
    // built-in structs don't have fields
    pub fields: Option<Vec<StructField>>,
    pub attribute: Attribute,
}

pub type StructField = FuncParam;

#[derive(Clone, Debug)]
pub struct StructInitField {
    pub name: InternedString,
    pub name_span: Span,
    pub value: Expr,
}

impl<'t, 's> Tokens<'t, 's> {
    pub fn parse_struct(&mut self) -> Result<Struct, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Struct))?.span;
        let (name, name_span) = self.pop_name_and_span(false /* allow_wildcard */)?;
        let mut generics = vec![];
        let mut generic_group_span = None;

        if let Some(Token { kind: TokenKind::Punct(Punct::Lt), span }) = self.peek() {
            generic_group_span = Some(*span);
            self.cursor += 1;
            generics = self.parse_generic_defs()?;
            let generic_span_end = self.match_and_pop(TokenKind::Punct(Punct::Gt))?.span;
            generic_group_span = generic_group_span.map(|span| span.merge(generic_span_end));
        }

        let fields = if let Some(Token { kind: TokenKind::Punct(Punct::Semicolon), .. }) = self.peek() {
            None
        } else {
            self.match_and_pop(TokenKind::Punct(Punct::Assign))?;

            let Token {
                kind: TokenKind::Group {
                    tokens: struct_body_tokens,
                    ..
                },
                span: struct_body_span,
            } = self.match_and_pop(TokenKind::Group { delim: Delim::Brace, tokens: vec![] })? else { unreachable!() };
            let mut struct_body_tokens = Tokens::new(struct_body_tokens, struct_body_span.end(), &self.intermediate_dir);
            Some(struct_body_tokens.parse_struct_fields()?)
        };

        self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;

        Ok(Struct {
            keyword_span,
            name,
            name_span,
            generics,
            generic_group_span,
            fields,
            attribute: Attribute::new(),
        })
    }

    pub fn parse_struct_fields(&mut self) -> Result<Vec<StructField>, Vec<Error>> {
        self.parse_func_params(false /* allow_wildcard */)
    }

    // NOTE: There must be at least 1 field!
    pub fn parse_struct_initialization(&mut self) -> Result<Vec<StructInitField>, Vec<Error>> {
        let mut fields = vec![];

        loop {
            let (name, name_span) = self.pop_name_and_span(false /* allow_wildcard */)?;
            self.match_and_pop(TokenKind::Punct(Punct::Colon))?;
            let value = self.parse_expr(true)?;
            fields.push(StructInitField {
                name,
                name_span,
                value,
            });

            match self.peek2() {
                (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), Some(_)) => {
                    self.cursor += 1;
                },
                (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), None) => {
                    return Ok(fields);
                },
                (Some(t), _) => {
                    return Err(vec![Error {
                        kind: ErrorKind::UnexpectedToken {
                            expected: ErrorToken::Ident,
                            got: (&t.kind).into(),
                        },
                        spans: t.span.simple_error(),
                        note: None,
                    }]);
                },
                (None, _) => {
                    return Ok(fields);
                },
            }
        }
    }

    /// `if p == Person { age: 30, name: "Bae" } { foo() }` is valid, but the parser cannot
    /// parse this. So the parser rejects this syntax instead of throwing very unreadable
    /// error message.
    pub fn check_ambiguous_struct_initialization(&mut self) -> Result<(), Vec<Error>> {
        match (self.peek_prev(), self.peek2()) {
            (
                Some(Token { kind: TokenKind::Ident(_), .. }),
                (
                    Some(Token { kind: TokenKind::Group { delim: Delim::Brace, .. }, span }),
                    Some(Token { kind: TokenKind::Group { delim: Delim::Brace, .. }, .. }),
                ),
            ) => Err(vec![Error {
                kind: ErrorKind::AmbiguousCurlyBraces,
                spans: span.simple_error(),
                note: None,
            }]),
            _ => Ok(()),
        }
    }
}
