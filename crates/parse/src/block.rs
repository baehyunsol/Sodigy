use crate::{Decl, Expr, ParseSession};
use sodigy_error::{Error, Warning};
use sodigy_keyword::Keyword;
use sodigy_token::{Token, TokenKind};

pub struct Block {
    decl: Vec<Decl>,
    value: Expr,
}

impl<'t> ParseSession<'t> {
    pub(crate) fn parse_block(&mut self) {
        match (self.tokens.get(self.cursor).map(|t| &t.kind), self.tokens.get(self.cursor + 1).map(|t| &t.kind)) {
            (Some(TokenKind::Keyword(Keyword::Let)), _) => {
                if let Ok(decl) = self.parse_decl() {
                    self.decls.push(decl);
                }
            },
            _ => todo!(),
        }
    }
}
