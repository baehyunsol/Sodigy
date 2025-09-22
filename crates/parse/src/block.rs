use crate::{Expr, Func, Let, ParseSession};
use sodigy_error::{Error, Warning};
use sodigy_keyword::Keyword;
use sodigy_token::{Token, TokenKind};

#[derive(Debug)]
pub struct Block {
    decls: Vec<Let>,
    funcs: Vec<Func>,
    value: Expr,
}

impl<'t> ParseSession<'t> {
    pub(crate) fn parse_block(&mut self) {
        match (self.tokens.get(self.cursor).map(|t| &t.kind), self.tokens.get(self.cursor + 1).map(|t| &t.kind)) {
            (Some(TokenKind::Keyword(Keyword::Let)), _) => {
                if let Ok(r#let) = self.parse_let() {
                    self.decls.push(r#let);
                }
            },
            (Some(TokenKind::Keyword(Keyword::Func)), _) => {
                if let Ok(func) = self.parse_func() {
                    self.funcs.push(func);
                }
            },
            t => panic!("TODO: {t:?}"),
        }
    }
}
