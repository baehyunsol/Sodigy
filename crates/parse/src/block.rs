use crate::{Expr, Func, Let, Tokens};
use sodigy_error::Error;
use sodigy_keyword::Keyword;
use sodigy_token::{ErrorToken, TokenKind};

#[derive(Debug)]
pub struct Block {
    lets: Vec<Let>,
    funcs: Vec<Func>,
    value: Expr,
}

impl<'t> Tokens<'t> {
    pub fn parse_block(&mut self) -> Result<Block, Vec<Error>> {
        let mut errors = vec![];
        let mut lets = vec![];
        let mut funcs = vec![];

        let value = loop {
            let (doc_comment, decorators) = match self.collect_doc_comment_and_decorators() {
                Ok((doc_comment, decorators)) => (doc_comment, decorators),

                // Even though there's an error, the parser can still distinguish decorators and declarations.
                // We'll continue parsing so that we can find more errors.
                Err(e) => {
                    errors.extend(e);
                    (None, vec![])
                },
            };

            match self.tokens.get(self.cursor).map(|t| &t.kind) {
                Some(TokenKind::Keyword(Keyword::Let)) => match self.parse_let() {
                    Ok(mut r#let) => {
                        r#let.doc_comment = doc_comment;
                        r#let.decorators = decorators;

                        lets.push(r#let);
                    },
                    Err(e) => {
                        errors.extend(e);
                        return Err(errors);
                    },
                },
                Some(TokenKind::Keyword(Keyword::Func)) => match self.parse_func() {
                    Ok(mut func) => {
                        func.doc_comment = doc_comment;
                        func.decorators = decorators;

                        funcs.push(func);
                    },
                    Err(e) => {
                        errors.extend(e);
                        return Err(errors);
                    },
                },
                Some(_) => {
                    if doc_comment.is_some() || !decorators.is_empty() {
                        // TODO: raise error
                        todo!()
                    }

                    match self.parse_expr() {
                        Ok(expr) => {
                            break expr;
                        },
                        Err(e) => {
                            errors.extend(e);
                            return Err(errors);
                        },
                    }
                },
                None => {
                    errors.push(self.unexpected_end(ErrorToken::Expr));
                    return Err(errors);
                },
            }
        };

        Ok(Block {
            lets,
            funcs,
            value,
        })
    }
}
