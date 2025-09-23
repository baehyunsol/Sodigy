use crate::{Decorator, DocComment, Expr, Tokens};
use sodigy_error::{Error, ErrorKind};
use sodigy_keyword::Keyword;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Delim, ErrorToken, Punct, Token, TokenKind};

#[derive(Debug)]
pub struct Func {
    name: InternedString,
    name_span: Span,
    args: Vec<Arg>,
    r#type: Option<Expr>,
    pub doc_comment: Option<DocComment>,
    pub decorators: Vec<Decorator>,
}

#[derive(Debug)]
pub struct Arg {
    name: InternedString,
    name_span: Span,
    r#type: Option<Expr>,
    pub doc_comment: Option<DocComment>,
    pub decorators: Vec<Decorator>,
}

impl<'t> Tokens<'t> {
    // `func foo(x) = 3;`
    // `func bar(x: Int, y: Int): Int = x + y;`
    pub fn parse_func(&mut self) -> Result<Func, Vec<Error>> {
        self.match_and_pop(TokenKind::Keyword(Keyword::Func))?;
        let (name, name_span) = self.pop_name_and_span()?;

        let arg_tokens = self.match_and_pop(TokenKind::Group { delim: Delim::Parenthesis, tokens: vec![] })?;
        let arg_tokens_inner = match &arg_tokens.kind {
            TokenKind::Group { tokens, .. } => tokens,
            _ => unreachable!(),
        };
        let mut arg_tokens = Tokens::new(arg_tokens_inner, arg_tokens.span.end());
        let args = arg_tokens.parse_func_arg_defs()?;

        let r#type = match self.tokens.get(self.cursor) {
            Some(Token { kind: TokenKind::Punct(Punct::Colon), ..}) => {
                self.cursor += 1;
                Some(self.parse_expr()?)
            },
            _ => None,
        };

        Ok(Func {
            name,
            name_span,
            args,
            r#type,

            // Its parent will set these fields.
            doc_comment: None,
            decorators: vec![],
        })
    }

    pub fn parse_func_arg_defs(&mut self) -> Result<Vec<Arg>, Vec<Error>> {
        let mut args = vec![];

        if self.peek().is_none() {
            return Ok(args);
        }

        'args: loop {
            let (doc_comment, decorators) = self.collect_doc_comment_and_decorators()?;
            let (name, name_span) = self.pop_name_and_span()?;
            let mut r#type = None;

            'colon_or_comma: loop {
                match self.tokens.get(self.cursor) {
                    Some(Token { kind: TokenKind::Punct(Punct::Colon), .. }) => {
                        self.cursor += 1;
                        r#type = Some(self.parse_expr()?);
                        continue 'colon_or_comma;
                    },
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }) | None => {
                        args.push(Arg {
                            name,
                            name_span,
                            r#type,
                            doc_comment,
                            decorators,
                        });

                        match self.tokens.get(self.cursor + 1) {
                            Some(_) => {
                                self.cursor += 1;
                                continue 'args;
                            },
                            None => {
                                break 'args;
                            },
                        }
                    },
                    Some(t) => {
                        return Err(vec![Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: ErrorToken::ColonOrComma,
                                got: (&t.kind).into(),
                            },
                            span: t.span,
                        }]);
                    },
                }
            }
        }

        Ok(args)
    }
}
