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
use sodigy_token::{Keyword, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Block {
    pub group_span: Span,
    pub lets: Vec<Let>,
    pub funcs: Vec<Func>,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,
    pub asserts: Vec<Assert>,
    pub aliases: Vec<Alias>,
    pub uses: Vec<Use>,

    // only the top-level block can have modules
    pub modules: Vec<Module>,

    // the top-level block doesn't have a value
    pub value: Box<Option<Expr>>,

    // only top-level block has an attribute
    pub attribute: Option<Attribute>,

    // Hir will lower a pipeline to a block.
    pub from_pipeline: bool,
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
            attribute: None,
            from_pipeline: false,
        }
    }

    // hir will use this function.
    pub fn iter_names(&self, is_top_level: bool) -> impl Iterator<Item = (InternedString, Span, NameKind)> {
        self.lets.iter().map(
            move |l| (
                l.name,
                l.name_span,
                if l.from_pipeline {
                    NameKind::Pipeline
                } else {
                    NameKind::Let { is_top_level }
                },
            )
        ).chain(
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

impl<'t, 's> Tokens<'t, 's> {
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

        // If it's top-level, it has to collect the module attribute.
        let attribute = if is_top_level {
            match self.collect_attribute(true) {
                Ok(attribute) => Some(attribute),
                Err(e) => {
                    errors = e;
                    None
                },
            }
        } else {
            None
        };

        loop {
            let attribute = match self.collect_attribute(false /* top_level */) {
                Ok(attribute) => attribute,

                // Even though there's an error, the parser can still find declarations.
                // We'll continue parsing so that we can find more errors.
                Err(e) => {
                    errors.extend(e);
                    Attribute::new()
                },
            };

            // FIXME: the same code is repeated multiple times...
            match self.peek2() {
                // `parse_let` might return multiple `Let`s because if there's a pattern,
                // it's destructured to multiple `Let`s.
                (Some(Token { kind: TokenKind::Keyword(Keyword::Let), .. }), _) => match self.parse_let() {
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
                // `impure \() ..` is an expression, but `impure fn ..` is an item.
                // So, we have to look 1 more token when we see `impure` keyword.
                (Some(Token { kind: TokenKind::Keyword(Keyword::Impure), .. }), Some(Token { kind: TokenKind::Keyword(Keyword::Fn), .. })) |
                (Some(Token { kind: TokenKind::Keyword(Keyword::Fn), .. }), _) => match self.parse_func() {
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
                (Some(Token { kind: TokenKind::Keyword(Keyword::Struct), .. }), _) => match self.parse_struct() {
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
                (Some(Token { kind: TokenKind::Keyword(Keyword::Enum), .. }), _) => match self.parse_enum() {
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
                (Some(Token { kind: TokenKind::Keyword(Keyword::Assert), .. }), _) => {
                    match self.parse_assert() {
                        Ok(mut assert) => {
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
                    }
                },
                (Some(Token { kind: TokenKind::Keyword(Keyword::Type), .. }), _) => match self. parse_alias() {
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
                (Some(Token { kind: TokenKind::Keyword(Keyword::Mod), .. }), _) => match self.parse_module() {
                    Ok(mut module) => {
                        module.attribute = attribute;
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
                (Some(Token { kind: TokenKind::Keyword(Keyword::Use), .. }), _) => match self.parse_use() {
                    Ok(mut uses_) => {
                        match (uses_.len(), attribute.is_empty()) {
                            (1, _) => {
                                uses_[0].attribute = attribute;
                            },
                            (_, true) => {},
                            (_, false) => {
                                // TODO: I'm not sure it's okay to naively distribute the attributes
                                for r#use in uses_.iter_mut() {
                                    r#use.attribute = attribute.clone();
                                }
                            },
                        }

                        uses.extend(uses_);
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
                (Some(t), _) => {
                    let initial_token = t.clone();

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
                        });
                        return Err(errors);
                    }

                    // TODO: throw an error if there's `pub` keyword

                    // If it's top-level, there shouldn't be an expr here. But we still parse
                    // the expr for the sake of better error message.
                    // If it's inline, there shouldn't be remaining tokens.
                    match self.parse_expr() {
                        Ok(expr) => {
                            if !is_top_level {
                                if let Some(t) = self.peek() {
                                    errors.push(Error {
                                        kind: ErrorKind::UnexpectedToken {
                                            expected: ErrorToken::Nothing,
                                            got: (&t.kind).into(),
                                        },
                                        spans: t.span.simple_error(),
                                        note: None,
                                    });
                                    return Err(errors);
                                }
                            }

                            else {
                                errors.push(Error {
                                    kind: ErrorKind::UnexpectedToken {
                                        expected: ErrorToken::Item,
                                        got: ErrorToken::Expr,
                                    },
                                    spans: expr.error_span_wide().simple_error(),
                                    note: None,
                                });
                                return Err(errors);
                            }

                            value = Some(expr);
                            break;
                        },
                        Err(e) => {
                            if is_top_level {
                                errors.push(Error {
                                    kind: ErrorKind::UnexpectedToken {
                                        expected: ErrorToken::Item,
                                        got: (&initial_token.kind).into(),
                                    },
                                    spans: initial_token.span.simple_error(),
                                    note: None,
                                });
                            }

                            else {
                                errors.extend(e);
                            }

                            return Err(errors);
                        },
                    }
                },
                (None, _) => {
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

        let result = Block {
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
            attribute,
            from_pipeline: false,
        };

        if errors.is_empty() {
            Ok(result)
        }

        // The items in `result` are all valid, so we'd better check the items so that
        // we can emit as many error messages as possible.
        // But since we discard `result` here, we have to check the `result` before we discard it.
        else {
            if let Err(more_errors) = result.check(is_top_level, &self.intermediate_dir) {
                errors.extend(more_errors);
            }

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
