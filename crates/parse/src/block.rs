use crate::{
    Enum,
    Expr,
    Func,
    Let,
    Module,
    Struct,
    Tokens,
    Use,
};
use sodigy_error::{Error, ErrorKind};
use sodigy_keyword::Keyword;
use sodigy_token::{ErrorToken, TokenKind};

#[derive(Clone, Debug)]
pub struct Block {
    pub lets: Vec<Let>,
    pub funcs: Vec<Func>,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,
    pub modules: Vec<Module>,
    pub uses: Vec<Use>,

    // top-level block doesn't have a value
    pub value: Box<Option<Expr>>,
}

impl<'t> Tokens<'t> {
    pub fn parse_block(
        &mut self,

        // top-level block doesn't have a value
        // also, there's a heuristic for top-level blocks: it continues parsing even
        // though there's an error so that it can find more errors
        top_level: bool,
    ) -> Result<Block, Vec<Error>> {
        let mut errors = vec![];
        let mut lets = vec![];
        let mut funcs = vec![];
        let mut structs = vec![];
        let mut enums = vec![];
        let mut modules = vec![];
        let mut uses = vec![];
        let mut value = None;

        loop {
            let (doc_comment, decorators) = match self.collect_doc_comment_and_decorators() {
                Ok((doc_comment, decorators)) => (doc_comment, decorators),

                // Even though there's an error, the parser can still distinguish decorators and declarations.
                // We'll continue parsing so that we can find more errors.
                Err(e) => {
                    errors.extend(e);
                    (None, vec![])
                },
            };

            // FIXME: the same code is repeated 4 times...
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Keyword(Keyword::Let)) => match self.parse_let() {
                    Ok(mut r#let) => {
                        r#let.doc_comment = doc_comment;
                        r#let.decorators = decorators;

                        lets.push(r#let);
                    },
                    Err(e) => {
                        errors.extend(e);

                        if top_level {
                            self.march_until_top_level_statement();
                        }

                        else {
                            return Err(errors);
                        }
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

                        if top_level {
                            self.march_until_top_level_statement();
                        }

                        else {
                            return Err(errors);
                        }
                    },
                },
                Some(TokenKind::Keyword(Keyword::Struct)) => match self.parse_struct() {
                    Ok(mut r#struct) => {
                        r#struct.doc_comment = doc_comment;
                        r#struct.decorators = decorators;

                        structs.push(r#struct);
                    },
                    Err(e) => {
                        errors.extend(e);

                        if top_level {
                            self.march_until_top_level_statement();
                        }

                        else {
                            return Err(errors);
                        }
                    },
                },
                Some(TokenKind::Keyword(Keyword::Enum)) => match self.parse_enum() {
                    Ok(mut r#enum) => {
                        r#enum.doc_comment = doc_comment;
                        r#enum.decorators = decorators;

                        enums.push(r#enum);
                    },
                    Err(e) => {
                        errors.extend(e);

                        if top_level {
                            self.march_until_top_level_statement();
                        }

                        else {
                            return Err(errors);
                        }
                    },
                },
                Some(TokenKind::Keyword(Keyword::Module)) => todo!(),
                Some(TokenKind::Keyword(Keyword::Use)) => todo!(),
                Some(t) => {
                    if top_level {
                        errors.push(Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: ErrorToken::Declaration,
                                got: t.into(),
                            },
                            span: self.peek().unwrap().span,
                            ..Error::default()
                        });
                        return Err(errors);
                    }

                    if doc_comment.is_some() || !decorators.is_empty() {
                        // TODO: raise error
                        todo!()
                    }

                    match self.parse_expr() {
                        Ok(expr) => {
                            value = Some(expr);
                            break;
                        },
                        Err(e) => {
                            errors.extend(e);
                            return Err(errors);
                        },
                    }
                },
                None => {
                    break;
                },
            }
        }

        if !top_level && value.is_none() {
            errors.push(Error {
                kind: ErrorKind::BlockWithoutValue,
                span: self.span_end,
                ..Error::default()
            });
        }

        if errors.is_empty() {
            Ok(Block {
                lets,
                funcs,
                structs,
                enums,
                modules,
                uses,
                value: Box::new(value),
            })
        }

        else {
            Err(errors)
        }
    }

    // If there's no top-level statement, it marches until the end
    fn march_until_top_level_statement(&mut self) {
        match self.peek().map(|t| &t.kind) {
            Some(TokenKind::Keyword(Keyword::Let | Keyword::Func | Keyword::Struct | Keyword::Enum)) => {
                return;
            },
            Some(_) => {
                self.cursor += 1;
            },
            None => {
                return;
            },
        }
    }
}
