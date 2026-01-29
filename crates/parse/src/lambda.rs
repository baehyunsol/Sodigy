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
    pub impure_keyword_span: Option<Span>,
    pub params: Vec<FuncParam>,
    pub param_group_span: Span,
    pub type_annot: Box<Option<Type>>,
    pub arrow_span: Span,
    pub value: Box<Expr>,
}

impl<'t, 's> Tokens<'t, 's> {
    pub fn parse_lambda(&mut self) -> Result<Lambda, Vec<Error>> {
        match self.peek() {
            Some(Token { kind: TokenKind::Group { delim: Delim::Lambda, tokens }, span }) => {
                let span = *span;
                let mut tokens = Tokens::new(tokens, span.end(), &self.intermediate_dir);
                let params = tokens.parse_func_params()?;
                self.cursor += 1;
                let mut type_annot = None;

                match self.peek() {
                    Some(Token { kind: TokenKind::Punct(Punct::ReturnType), .. }) => {
                        self.cursor += 1;
                        type_annot = Some(self.parse_type()?);
                    },
                    _ => {},
                }

                let arrow_span = self.match_and_pop(TokenKind::Punct(Punct::Arrow))?.span;
                let value = self.parse_expr(true)?;

                Ok(Lambda {
                    // if there's `impure` keyword, its callee will change these values.
                    is_pure: true,
                    impure_keyword_span: None,

                    params,
                    param_group_span: span,
                    type_annot: Box::new(type_annot),
                    arrow_span,
                    value: Box::new(value),
                })
            },
            Some(t) => {
                return Err(vec![Error {
                    kind: ErrorKind::UnexpectedToken {
                        expected: ErrorToken::Group(Delim::Lambda),
                        got: (&t.kind).into(),
                    },
                    spans: t.span.simple_error(),
                    note: None,
                }]);
            },
            None => {
                return Err(vec![self.unexpected_end((&TokenKind::Group { delim: Delim::Lambda, tokens: vec![] }).into())]);
            },
        }
    }
}
