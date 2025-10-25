use crate::{
    Alias,
    Assert,
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
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_name_analysis::NameKind;
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use sodigy_token::{Keyword, Punct, TokenKind};

#[derive(Clone, Debug)]
pub struct Block {
    pub group_span: Span,
    pub lets: Vec<Let>,
    pub funcs: Vec<Func>,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,
    pub asserts: Vec<Assert>,
    pub aliases: Vec<Alias>,

    // only the top-level block can have modules
    pub modules: Vec<Module>,
    pub uses: Vec<Use>,

    // the top-level block doesn't have a value
    pub value: Box<Option<Expr>>,
}

impl Block {
    // It may or may not be a valid `Block`.
    // This is only for unintialized or erroneous `Session`.
    pub fn dummy() -> Self {
        Block {
            group_span: Span::None,
            lets: vec![],
            funcs: vec![],
            structs: vec![],
            enums: vec![],
            asserts: vec![],
            aliases: vec![],
            modules: vec![],
            uses: vec![],
            value: Box::new(None),
        }
    }

    // hir will use this function.
    pub fn iter_names(&self, is_top_level: bool) -> impl Iterator<Item = (InternedString, Span, NameKind)> {
        self.lets.iter().map(move |l| (l.name, l.name_span, NameKind::Let { is_top_level })).chain(
            self.funcs.iter().map(|f| (f.name, f.name_span, NameKind::Func))
        ).chain(
            self.structs.iter().map(|s| (s.name, s.name_span, NameKind::Struct))
        ).chain(
            self.enums.iter().map(|e| (e.name, e.name_span, NameKind::Enum))
        ).chain(
            self.aliases.iter().map(|a| (a.name, a.name_span, NameKind::Alias))
        ).chain(
            self.modules.iter().map(|m| (m.name, m.name_span, NameKind::Module))
        ).chain(
            self.uses.iter().map(|u| (u.name, u.name_span, NameKind::Use))
        )
    }
}

impl<'t> Tokens<'t> {
    pub fn parse_block(
        &mut self,

        // top-level block doesn't have a value
        // also, there's a heuristic for top-level blocks: it continues parsing even
        // though there's an error so that it can find more errors
        is_top_level: bool,
        group_span: Span,
    ) -> Result<Block, Vec<Error>> {
        let mut errors = vec![];
        let mut lets = vec![];
        let mut funcs = vec![];
        let mut structs = vec![];
        let mut enums = vec![];
        let mut asserts = vec![];
        let mut aliases = vec![];
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
                // `parse_let` might return multiple `Let`s because if there's a pattern,
                // it's destructured to multiple `Let`s.
                Some(TokenKind::Keyword(Keyword::Let)) => match self.parse_let() {
                    Ok(mut lets_) => {
                        match (lets_.len(), attribute.is_empty()) {
                            (1, _) => {
                                lets_[0].attribute = attribute;
                            },
                            (_, true) => {},

                            // How should I attach attributes to the destructured lets?
                            (_, false) => todo!(),
                        }

                        lets.extend(lets_);
                    },
                    Err(e) => {
                        errors.extend(e);

                        if is_top_level {
                            self.march_until_top_level_statement();
                        }

                        else {
                            return Err(errors);
                        }
                    },
                },
                Some(TokenKind::Keyword(Keyword::Fn)) => match self.parse_func() {
                    Ok(mut func) => {
                        func.attribute = attribute;
                        funcs.push(func);
                    },
                    Err(e) => {
                        errors.extend(e);

                        if is_top_level {
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

                        if is_top_level {
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

                        if is_top_level {
                            self.march_until_top_level_statement();
                        }

                        else {
                            return Err(errors);
                        }
                    },
                },
                Some(TokenKind::Keyword(Keyword::Assert)) => match self.parse_assert() {
                    Ok(mut assert) => {
                        if let Some(doc_comment) = &attribute.doc_comment {
                            errors.push(Error {
                                kind: ErrorKind::DocCommentNotAllowed,
                                spans: vec![
                                    RenderableSpan {
                                        span: assert.keyword_span,
                                        auxiliary: false,
                                        note: Some(String::from("This assertion is documented by the doc comment.")),
                                    },
                                    RenderableSpan {
                                        span: doc_comment.0.last().unwrap().marker_span,
                                        auxiliary: true,
                                        note: Some(String::from("This doc comment is documenting the assertion.")),
                                    },
                                ],
                                note: Some(String::from("If you want to add a note, use `@note` decorator.")),
                            });
                        }

                        assert.attribute = attribute;
                        asserts.push(assert);
                    },
                    Err(e) => {
                        errors.extend(e);

                        if is_top_level {
                            self.march_until_top_level_statement();
                        }

                        else {
                            return Err(errors);
                        }
                    },
                },
                Some(TokenKind::Keyword(Keyword::Type)) => match self. parse_alias() {
                    Ok(mut alias) => {
                        alias.attribute = attribute;
                        aliases.push(alias);
                    },
                    Err(e) => {
                        errors.extend(e);

                        if is_top_level {
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

                        if is_top_level {
                            self.march_until_top_level_statement();
                        }

                        else {
                            return Err(errors);
                        }
                    },
                },
                Some(TokenKind::Keyword(Keyword::Use)) => todo!(),
                Some(t) => {
                    if is_top_level {
                        let note = match t {
                            // There's a very weird edge case: If the tokens are `<Decorator> -> <DocComment> -> <Semicolon> -> <Expr>`,
                            // you'll see this error message with the semicolon.
                            TokenKind::Punct(Punct::Semicolon) if !attribute.decorators.is_empty() => Some(String::from(
                                "Don't put a semicolon after a decorator."
                            )),
                            _ => None,
                        };

                        errors.push(Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: ErrorToken::Declaration,
                                got: t.into(),
                            },
                            spans: self.peek().unwrap().span.simple_error(),
                            note,
                        });
                        return Err(errors);
                    }

                    if let Some(doc_comment) = &attribute.doc_comment {
                        errors.push(Error {
                            kind: ErrorKind::DocCommentNotAllowed,
                            spans: vec![
                                RenderableSpan {
                                    span: doc_comment.0[0].marker_span,
                                    auxiliary: false,
                                    note: Some(String::from("This doc comment is documenting the expression.")),
                                },
                                RenderableSpan {
                                    span: self.peek().unwrap().span.begin(),
                                    auxiliary: true,
                                    note: Some(String::from("This expression is documented by the doc comment.")),
                                },
                            ],
                            note: Some(String::from("You can't add a document for an expression.")),
                            ..Error::default()
                        });
                        return Err(errors);
                    }

                    if let Some(decorator) = attribute.decorators.get(0) {
                        errors.push(Error {
                            kind: ErrorKind::DecoratorNotAllowed,
                            spans: vec![
                                RenderableSpan {
                                    span: decorator.name_span,
                                    auxiliary: false,
                                    note: Some(String::from("This decorator is decorating the expression.")),
                                },
                                RenderableSpan {
                                    span: self.peek().unwrap().span.begin(),
                                    auxiliary: true,
                                    note: Some(String::from("This expression is decorated by the decorator.")),
                                },
                            ],
                            note: Some(String::from("You can't decorate an expression.")),
                            ..Error::default()
                        });
                        return Err(errors);
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

        if !is_top_level && value.is_none() {
            errors.push(Error {
                kind: ErrorKind::BlockWithoutValue,
                spans: self.span_end.simple_error(),
                note: None,
            });
        }

        if errors.is_empty() {
            Ok(Block {
                group_span,
                lets,
                funcs,
                structs,
                enums,
                asserts,
                aliases,
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
                Some(TokenKind::Keyword(
                    Keyword::Let | Keyword::Fn |
                    Keyword::Struct | Keyword::Enum |
                    Keyword::Type | Keyword::Assert
                )) => {
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
