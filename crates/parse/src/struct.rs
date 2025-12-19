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
    pub fields: Vec<StructField>,
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
        let (name, name_span) = self.pop_name_and_span()?;
        let mut generics = vec![];
        let mut generic_group_span = None;

        if let Some(Token { kind: TokenKind::Punct(Punct::Lt), span }) = self.peek() {
            generic_group_span = Some(*span);
            self.cursor += 1;
            generics = self.parse_generic_defs()?;
            let generic_span_end = self.match_and_pop(TokenKind::Punct(Punct::Gt))?.span;
            generic_group_span = generic_group_span.map(|span| span.merge(generic_span_end));
        }

        self.match_and_pop(TokenKind::Punct(Punct::Assign))?;

        let Token {
            kind: TokenKind::Group {
                tokens: struct_body_tokens,
                ..
            },
            span: struct_body_span,
        } = self.match_and_pop(TokenKind::Group { delim: Delim::Brace, tokens: vec![] })? else { unreachable!() };
        let mut struct_body_tokens = Tokens::new(struct_body_tokens, struct_body_span.end(), &self.intermediate_dir);
        let fields = struct_body_tokens.parse_struct_fields()?;
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
        self.parse_func_params()
    }

    // In Sodigy, curly braces following an identifier can be either
    // 1. a struct initialization, like `foo { x: 3, y: 4 }`
    // 2. an if branch, like `if foo { 4 } else { 5 }`
    //
    // It's impossible to perfectly distinguish the two. So it uses some kinda heuristic.
    // If the inner tokens start with an identifier and followed by a colon, it treats it
    // as a struct initialization.
    //
    // `foo { x: 3, y: 4 }` -> `Some(Ok([("x", 3), ("y", 4)]))`
    // `foo { x: 3, y }` -> `Some(Err("expected colon, got nothing"))`
    // `foo { x }` -> `None`
    // `foo {}` -> `Some(Err("an empty curly brace block"))`
    //    -> well... this is ambiguous. we're not sure whether the programmer intended a struct initialization or an if branch
    //    -> in Sodigy, an empty curly brace block is not allowed in any context due to this reason
    //       -> so that we can throw a less ambiguous error messsage "an empty curly brace group!"
    //    -> also, this is the reason why Sodigy doesn't allow a struct without any fields
    pub fn try_parse_struct_initialization(&mut self) -> Option<Result<Vec<StructInitField>, Vec<Error>>> {
        match self.peek2() {
            (
                Some(Token { kind: TokenKind::Ident(_), .. }),
                Some(Token { kind: TokenKind::Punct(Punct::Colon), .. }),
            ) => {},
            (None, None) => {
                return Some(Err(vec![Error {
                    kind: ErrorKind::EmptyCurlyBraceBlock,
                    spans: self.span_end.simple_error(),
                    note: None,
                }]));
            },
            _ => {
                return None;
            },
        }

        Some(self.parse_struct_initialization())
    }

    // NOTE: There must be at least 1 field!
    pub fn parse_struct_initialization(&mut self) -> Result<Vec<StructInitField>, Vec<Error>> {
        let mut fields = vec![];

        loop {
            let (name, name_span) = self.pop_name_and_span()?;
            self.match_and_pop(TokenKind::Punct(Punct::Colon))?;
            let value = self.parse_expr()?;
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
}
