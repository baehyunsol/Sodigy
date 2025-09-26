use crate::{
    Attribute,
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
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{ErrorToken, TokenKind};

#[derive(Clone, Debug)]
pub struct Block {
    pub lets: Vec<Let>,
    pub funcs: Vec<Func>,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,

    // only the top-level block can have modules
    pub modules: Vec<Module>,
    pub uses: Vec<Use>,

    // the top-level block doesn't have a value
    pub value: Box<Option<Expr>>,
}

impl Block {
    // hir will use this function.
    pub fn iter_names(&self) -> impl Iterator<Item = (InternedString, Span)> {
        self.lets.iter().map(|l| (l.name, l.name_span)).chain(
            self.funcs.iter().map(|f| (f.name, f.name_span))
        ).chain(
            self.structs.iter().map(|s| (s.name, s.name_span))
        ).chain(
            self.enums.iter().map(|e| (e.name, e.name_span))
        ).chain(
            self.modules.iter().map(|m| (m.name, m.name_span))
        ).chain(
            self.uses.iter().map(|u| (u.name, u.name_span))
        )
    }
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
            let attribute = match self.collect_attribute() {
                Ok(attribute) => attribute,

                // Even though there's an error, the parser can still find declarations.
                // We'll continue parsing so that we can find more errors.
                Err(e) => {
                    errors.extend(e);
                    Attribute::new()
                },
            };

            // FIXME: the same code is repeated multiple times...
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Keyword(Keyword::Let)) => match self.parse_let() {
                    Ok(mut r#let) => {
                        r#let.attribute = attribute;
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
                        func.attribute = attribute;
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
                        r#struct.attribute = attribute;
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
                        r#enum.attribute = attribute;
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
                Some(TokenKind::Keyword(Keyword::Module)) => match self.parse_module() {
                    Ok(module) => {
                        if !attribute.is_empty() {
                            // TODO: raise error
                            todo!()
                        }

                        modules.push(module);
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

                    if !attribute.is_empty() {
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
        loop {
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
}
