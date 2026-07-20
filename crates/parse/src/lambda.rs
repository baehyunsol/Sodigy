use crate::{
    Expr,
    FuncParam,
    Tokens,
    Type,
};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_span::Span;
use sodigy_token::{Delim, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Lambda {
    pub is_pure: bool,
    pub proc_keyword_span: Option<Span>,
    pub backslash_span: Span,
    pub params: Vec<FuncParam>,
    pub param_group_span: Span,
    pub type_annot: Box<Option<Type>>,
    pub arrow_span: Span,
    pub value: Box<Expr>,
}

impl<'t, 's> Tokens<'t, 's> {
    // The cursor must be pointing to the backslash character.
    // If there's `proc` keyword before the backslash, its callee will take care of that.
    pub fn parse_lambda(&mut self) -> Result<Lambda, Vec<Error>> {
        match self.peek2() {
            (Some(Token { kind: TokenKind::Punct(Punct::Backslash), .. }), Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, span })) |
            (Some(Token { kind: TokenKind::Group { delim: Delim::Lambda, tokens }, span }), _) => {
                let (backslash_span, param_group_span, jump) = match self.peek2() {
                    (Some(Token { kind: TokenKind::Punct(_), span: span1 }), Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, .. }, span: span2 })) => (span1.clone(), span1.merge(span2), 2),
                    (Some(Token { kind: TokenKind::Group { delim: Delim::Lambda, .. }, span }), _) => (span.start(), span.clone(), 1),
                    _ => unreachable!(),
                };

                let mut tokens = Tokens::new(tokens, param_group_span.end(), false, self.intermediate_dir);
                let params = tokens.parse_func_params(true /* allow_wildcard */)?;
                self.cursor += jump;
                let mut type_annot = None;

                match self.peek() {
                    Some(Token { kind: TokenKind::Punct(Punct::ReturnType), .. }) => {
                        self.cursor += 1;
                        type_annot = Some(self.parse_type()?);
                    },
                    _ => {},
                }

                let arrow_span = self.match_and_pop(TokenKind::Punct(Punct::Arrow))?.span.clone();
                let value = self.parse_expr(true)?;

                Ok(Lambda {
                    // if there's `proc` keyword, its callee will change these values.
                    is_pure: true,
                    proc_keyword_span: None,
                    backslash_span,

                    params,
                    param_group_span,
                    type_annot: Box::new(type_annot),
                    arrow_span,
                    value: Box::new(value),
                })
            },
            (Some(Token { kind: TokenKind::Punct(Punct::Backslash), span }), _) => {
                let backslash_span = span.clone();
                let mut param_group_span_start = span.clone();
                let mut arrow_span = None;
                self.cursor += 1;

                let mut param_tokens_start_index = self.cursor;
                let mut param_tokens_end_index = None;

                for (i, token) in self.enumerate_forward() {
                    param_tokens_end_index = Some(i);

                    if let Token { kind: TokenKind::Punct(Punct::ReturnType), span } = token {
                        // This is a syntax error. I won't allow users to annotate type in this way.
                        // They must use a parenthesis.
                        todo!();
                    }

                    else if let Token { kind: TokenKind::Punct(Punct::Arrow), span } = token {
                        arrow_span = Some(span.clone());
                        break;
                    }
                }

                match (arrow_span, param_tokens_end_index) {
                    (Some(arrow_span), Some(param_tokens_end_index)) => {
                        let end_span = self.tokens[param_tokens_end_index - 1].span.end();
                        let param_group_span = param_group_span_start.merge(&end_span);
                        let mut tokens = Tokens::new(&self.tokens[param_tokens_start_index..param_tokens_end_index], end_span, false, self.intermediate_dir);
                        let params = tokens.parse_func_params(true /* allow_wildcard */)?;
                        self.cursor = param_tokens_end_index;

                        let arrow_span = self.match_and_pop(TokenKind::Punct(Punct::Arrow))?.span.clone();
                        let value = self.parse_expr(true)?;

                        Ok(Lambda {
                            // if there's `proc` keyword, its callee will change these values.
                            is_pure: true,
                            proc_keyword_span: None,
                            backslash_span,

                            params,
                            param_group_span,
                            type_annot: Box::new(None),
                            arrow_span,
                            value: Box::new(value),
                        })
                    },
                    _ => todo!(),
                }
            },
            (Some(t), _) => {
                return Err(vec![Error {
                    kind: ErrorKind::UnexpectedToken {
                        expected: ErrorToken::Group(Delim::Lambda),
                        got: (&t.kind).into(),
                    },
                    spans: t.span.simple_error(),
                    note: None,
                }]);
            },
            (None, _) => {
                return Err(vec![self.unexpected_end((&TokenKind::Group { delim: Delim::Lambda, tokens: vec![] }).into())]);
            },
        }
    }
}
