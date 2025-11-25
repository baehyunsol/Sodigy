use crate::{
    Alias,
    Assert,
    Enum,
    EnumVariant,
    Expr,
    Func,
    Generic,
    Let,
    Module,
    Session,
    Struct,
    Use,
};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_name_analysis::{IdentWithOrigin, NameOrigin};
use sodigy_parse::{self as ast, DocComment};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::{
    InternedString,
    intern_string,
    unintern_string,
};
use std::collections::hash_map::{Entry, HashMap};

impl Session {
    pub fn lower_attribute(
        &mut self,
        ast_attribute: &ast::Attribute,
        kind: AttributeKind,
        keyword_span: Span,
        is_top_level: bool,
    ) -> Result<Attribute, ()> {
        let attribute_rule_key = AttributeRuleKey {
            kind,
            is_top_level,
            is_std: self.is_std,
        };
        let attribute_rule = match self.attribute_rule_cache.get(&attribute_rule_key) {
            Some(rule) => rule.clone(),
            None => {
                let rule = match kind {
                    AttributeKind::Alias => Alias::get_attribute_rule(is_top_level, self.is_std, self),
                    AttributeKind::Assert => Assert::get_attribute_rule(is_top_level, self.is_std, self),
                    AttributeKind::Enum => Enum::get_attribute_rule(is_top_level, self.is_std, self),
                    AttributeKind::EnumVariant => EnumVariant::get_attribute_rule(is_top_level, self.is_std, self),
                    AttributeKind::Func => Func::get_attribute_rule(is_top_level, self.is_std, self),
                    AttributeKind::Let => Let::get_attribute_rule(is_top_level, self.is_std, self),
                    AttributeKind::Module => Module::get_attribute_rule(is_top_level, self.is_std, self),
                    AttributeKind::Struct => Struct::get_attribute_rule(is_top_level, self.is_std, self),
                    AttributeKind::Use => Use::get_attribute_rule(is_top_level, self.is_std, self),
                };
                self.attribute_rule_cache.insert(attribute_rule_key, rule.clone());
                rule
            },
        };

        Attribute::from_ast(ast_attribute, self, &attribute_rule, keyword_span)
    }

    pub fn collect_lang_items(
        &mut self,
        attribute: &Attribute,
        lang_item_span: Span,
        generic_defs: Option<&[Generic]>,
    ) -> Result<(), ()> {
        if let Some(lang_item) = attribute.lang_item(&self.intermediate_dir) {
            self.lang_items.insert(lang_item, lang_item_span);
        }

        if let Some(lang_item_generics) = attribute.lang_item_generics(&self.intermediate_dir) {
            if let Some(generic_defs) = generic_defs {
                if lang_item_generics.len() == generic_defs.len() {
                    for i in 0..generic_defs.len() {
                        self.lang_items.insert(lang_item_generics[i].to_string(), generic_defs[i].name_span);
                    }
                }

                else {
                    // What kinda error should it throw?
                    todo!()
                }
            }

            else {
                // What kinda error should it throw?
                todo!()
            }
        }

        Ok(())
    }
}

// `ast::Attribute` is first lowered to this type. It does some basic
// checks (redundant names, undefined names, arguments).
// Each item extracts extra information from this type.
#[derive(Clone, Debug)]
pub struct Attribute {
    pub doc_comment: Option<DocComment>,
    pub decorators: HashMap<InternedString, Decorator>,
    pub visibility: Visibility,
}

impl Attribute {
    pub fn new() -> Attribute {
        Attribute {
            doc_comment: None,
            decorators: HashMap::new(),
            visibility: Visibility::private(),
        }
    }

    pub fn from_ast(
        ast_attribute: &ast::Attribute,
        session: &mut Session,
        rule: &AttributeRule,

        // span of `fn`, `let`, `enum`, `struct`, ...
        item_keyword_span: Span,
    ) -> Result<Attribute, ()> {
        let mut has_error = false;
        let doc_comment = match (rule.doc_comment, &ast_attribute.doc_comment) {
            (Requirement::Must, None) => {
                has_error = true;
                session.errors.push(Error {
                    kind: ErrorKind::MissingDocComment,
                    spans: item_keyword_span.simple_error(),
                    note: rule.doc_comment_error_note.clone(),
                });
                None
            },
            (Requirement::Never, Some(doc_comment)) => {
                has_error = true;
                session.errors.push(Error {
                    kind: ErrorKind::DocCommentNotAllowed,
                    spans: vec![
                        RenderableSpan {
                            span: item_keyword_span,
                            auxiliary: true,
                            note: Some(String::from("You can't add doc comment to this.")),
                        },
                        RenderableSpan {
                            span: doc_comment.0[0].marker_span,
                            auxiliary: false,
                            note: None,
                        },
                    ],
                    note: rule.doc_comment_error_note.clone(),
                });
                None
            },
            _ => ast_attribute.doc_comment.clone(),
        };
        let visibility = match (rule.visibility, &ast_attribute.visibility) {
            (Requirement::Must, None) => {
                has_error = true;
                session.errors.push(Error {
                    kind: ErrorKind::MissingVisibility,
                    spans: item_keyword_span.simple_error(),
                    note: rule.visibility_error_note.clone(),
                });
                Visibility::private()
            },
            (Requirement::Never, Some(ast_visibility)) => {
                has_error = true;
                session.errors.push(Error {
                    kind: ErrorKind::CannotBePublic,
                    spans: vec![
                        RenderableSpan {
                            span: item_keyword_span,
                            auxiliary: true,
                            note: Some(String::from("This cannot be public.")),
                        },
                        RenderableSpan {
                            span: ast_visibility.keyword_span,
                            auxiliary: false,
                            note: None,
                        },
                    ],
                    note: rule.visibility_error_note.clone(),
                });

                match Visibility::from_ast(&ast_visibility, session) {
                    Ok(visibility) => visibility,
                    Err(()) => {
                        has_error = true;
                        Visibility::private()
                    },
                }
            },
            (_, None) => Visibility::private(),
            (_, Some(ast_visibility)) => match Visibility::from_ast(&ast_visibility, session) {
                Ok(visibility) => visibility,
                Err(()) => {
                    has_error = true;
                    Visibility::private()
                },
            },
        };

        let mut decorators = HashMap::with_capacity(ast_attribute.decorators.len());

        // for error messages
        let mut spans_by_name: HashMap<InternedString, Vec<Span>> = HashMap::new();

        for ast_decorator in ast_attribute.decorators.iter() {
            match rule.decorators.get(&ast_decorator.name) {
                Some(rule) => {
                    if let Requirement::Never = rule.requirement {
                        has_error = true;
                        session.errors.push(Error {
                            kind: ErrorKind::UnexpectedDecorator(ast_decorator.name),
                            spans: ast_decorator.name_span.simple_error(),
                            note: None,
                        });
                    }

                    match (rule.arg_requirement, &ast_decorator.args) {
                        (Requirement::Must, None) => {
                            has_error = true;
                            session.errors.push(Error {
                                kind: ErrorKind::MissingDecoratorArgument {
                                    expected: 1,  // how many?
                                    got: 0,
                                },
                                spans: ast_decorator.name_span.simple_error(),
                                note: None,
                            });
                        },
                        (Requirement::Never, Some(ast_args)) => {
                            has_error = true;
                            session.errors.push(Error {
                                kind: ErrorKind::UnexpectedDecoratorArgument {
                                    expected: 0,
                                    got: ast_args.len(),
                                },
                                spans: vec![
                                    RenderableSpan {
                                        span: ast_decorator.name_span,
                                        auxiliary: true,
                                        note: Some(String::from("It requires no arguments.")),
                                    },
                                    RenderableSpan {
                                        span: ast_decorator.arg_group_span.unwrap(),
                                        auxiliary: false,
                                        note: Some(String::from("Remove this parenthesis.")),
                                    },
                                ],
                                note: None,
                            });
                        },
                        (_, Some(ast_args)) => {
                            let mut keyword_args: HashMap<InternedString, Expr> = HashMap::new();
                            let mut positional_args: Vec<&ast::Expr> = vec![];
                            let mut spans_by_keyword: HashMap<InternedString, Vec<Span>> = HashMap::new();

                            for ast_arg in ast_args.iter() {
                                match ast_arg.keyword {
                                    Some((keyword, span)) => match rule.keyword_args.get(&keyword) {
                                        Some(KeywordArgRule {
                                            requirement,
                                            requirement_error_note,
                                            arg_type,
                                            arg_type_error_note,
                                        }) => {
                                            if let Requirement::Never = requirement {
                                                has_error = true;
                                                session.errors.push(Error {
                                                    kind: ErrorKind::InvalidKeywordArgument(keyword),
                                                    spans: span.simple_error(),
                                                    note: requirement_error_note.clone(),
                                                });
                                            }

                                            match spans_by_keyword.entry(keyword) {
                                                Entry::Occupied(mut e) => {
                                                    e.get_mut().push(span);
                                                },
                                                Entry::Vacant(e) => {
                                                    e.insert(vec![span]);
                                                },
                                            }

                                            match Expr::from_ast(&ast_arg.arg, session) {
                                                Ok(arg) => match check_arg_type(&arg, *arg_type, arg_type_error_note, session) {
                                                    Ok(()) => {
                                                        keyword_args.insert(keyword, arg);
                                                    },
                                                    Err(()) => {
                                                        has_error = true;
                                                    },
                                                },
                                                Err(()) => {
                                                    has_error = true;
                                                },
                                            }
                                        },
                                        None => {
                                            has_error = true;
                                            session.errors.push(Error {
                                                kind: ErrorKind::InvalidKeywordArgument(keyword),
                                                spans: span.simple_error(),
                                                note: None,
                                            });
                                        },
                                    },
                                    None => {
                                        positional_args.push(&ast_arg.arg);
                                    },
                                }
                            }

                            for (keyword, spans) in spans_by_keyword.iter() {
                                if spans.len() > 1 {
                                    has_error = true;
                                    session.errors.push(Error {
                                        kind: ErrorKind::KeywordArgumentRepeated(*keyword),
                                        spans: spans.iter().map(
                                            |span| RenderableSpan {
                                                span: *span,
                                                auxiliary: false,
                                                note: None,
                                            }
                                        ).collect(),
                                        note: None,
                                    });
                                }
                            }

                            for (keyword, KeywordArgRule { requirement, requirement_error_note, .. }) in rule.keyword_args.iter() {
                                if let Requirement::Must = requirement {
                                    if spans_by_keyword.get(keyword).is_none() {
                                        session.errors.push(Error {
                                            kind: ErrorKind::MissingKeywordArgument(*keyword),
                                            spans: ast_decorator.name_span.simple_error(),
                                            note: requirement_error_note.clone(),
                                        });
                                    }
                                }
                            }

                            let count_rule = match (rule.arg_count, positional_args.len()) {
                                (ArgCount::Zero, 1..) => Err((
                                    ErrorKind::UnexpectedDecoratorArgument {
                                        expected: 0,
                                        got: positional_args.len(),
                                    },
                                    positional_args.iter().map(
                                        |arg| RenderableSpan {
                                            span: arg.error_span(),
                                            auxiliary: false,
                                            note: None,
                                        }
                                    ).collect(),
                                )),
                                (ArgCount::Eq(n), m) if n > m => Err((
                                    ErrorKind::MissingDecoratorArgument {
                                        expected: n,
                                        got: m,
                                    },
                                    ast_decorator.name_span.simple_error(),
                                )),
                                (ArgCount::Eq(n), m) if n < m => Err((
                                    ErrorKind::UnexpectedDecoratorArgument {
                                        expected: n,
                                        got: m,
                                    },
                                    positional_args[n..].iter().map(
                                        |arg| RenderableSpan {
                                            span: arg.error_span(),
                                            auxiliary: false,
                                            note: None,
                                        }
                                    ).collect(),
                                )),
                                (ArgCount::Gt(n), m) if n >= m => Err((
                                    ErrorKind::MissingDecoratorArgument {
                                        expected: n + 1,
                                        got: m,
                                    },
                                    ast_decorator.name_span.simple_error(),
                                )),
                                (ArgCount::Lt(n), m) if n <= m => Err((
                                    ErrorKind::UnexpectedDecoratorArgument {
                                        expected: n - 1,
                                        got: m,
                                    },
                                    positional_args[(n - 1)..].iter().map(
                                        |arg| RenderableSpan {
                                            span: arg.error_span(),
                                            auxiliary: false,
                                            note: None,
                                        }
                                    ).collect(),
                                )),
                                _ => Ok(()),
                            };

                            match count_rule {
                                Ok(()) => {
                                    let mut args = Vec::with_capacity(positional_args.len());

                                    for ast_arg in positional_args.iter() {
                                        match Expr::from_ast(ast_arg, session) {
                                            Ok(arg) => match check_arg_type(&arg, rule.arg_type, &rule.arg_type_error_note, session) {
                                                Ok(()) => {
                                                    args.push(arg);
                                                },
                                                Err(()) => {
                                                    has_error = true;
                                                },
                                            },
                                            Err(()) => {
                                                has_error = true;
                                            },
                                        }
                                    }

                                    decorators.insert(
                                        ast_decorator.name,
                                        Decorator {
                                            name: ast_decorator.name,
                                            name_span: ast_decorator.name_span,
                                            args,
                                            keyword_args,
                                        },
                                    );
                                },
                                Err((error_kind, error_span)) => {
                                    has_error = true;
                                    session.errors.push(Error {
                                        kind: error_kind,
                                        spans: error_span,
                                        note: rule.arg_count_error_note.clone(),
                                    });
                                },
                            }
                        },
                        (_, None) => {
                            decorators.insert(
                                ast_decorator.name,
                                Decorator {
                                    name: ast_decorator.name,
                                    name_span: ast_decorator.name_span,
                                    args: vec![],
                                    keyword_args: HashMap::new(),
                                },
                            );
                        },
                    }
                },
                None => {
                    // TODO: try `rule.decorators.get(&name[..i])` to generate a better error message
                    has_error = true;
                    session.errors.push(Error {
                        kind: ErrorKind::InvalidDecorator(ast_decorator.name),
                        spans: ast_decorator.name_span.simple_error(),
                        note: None,
                    });
                },
            }

            match spans_by_name.entry(ast_decorator.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(ast_decorator.name_span);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![ast_decorator.name_span]);
                },
            }
        }

        for (name, spans) in spans_by_name.iter() {
            if spans.len() > 1 {
                has_error = true;
                session.errors.push(Error {
                    kind: ErrorKind::RedundantDecorator(*name),
                    spans: spans.iter().map(
                        |span| RenderableSpan {
                            span: *span,
                            auxiliary: false,
                            note: None,
                        }
                    ).collect(),
                    note: None,
                });
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(Attribute {
                doc_comment,
                decorators,
                visibility,
            })
        }
    }

    pub fn get_decorator<'a>(&'a self, key: &[u8], intermediate_dir: &str) -> Option<&'a Decorator> {
        self.decorators.get(&intern_string(key, intermediate_dir).unwrap())
    }

    pub fn lang_item(&self, intermediate_dir: &str) -> Option<String> {
        match self.decorators.get(&intern_string(b"lang_item", intermediate_dir).unwrap()) {
            Some(d) => match d.args.get(0) {
                Some(Expr::String { s, .. }) => Some(String::from_utf8_lossy(&unintern_string(*s, intermediate_dir).unwrap().unwrap()).to_string()),
                _ => unreachable!(),
            },
            None => None,
        }
    }

    pub fn lang_item_generics(&self, intermediate_dir: &str) -> Option<Vec<String>> {
        match self.decorators.get(&intern_string(b"lang_item_generics", intermediate_dir).unwrap()) {
            Some(d) => Some(d.args.iter().map(
                |arg| match arg {
                    Expr::String { s, .. } => String::from_utf8_lossy(&unintern_string(*s, intermediate_dir).unwrap().unwrap()).to_string(),
                    _ => unreachable!(),
                }
            ).collect()),
            None => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AttributeRule {
    pub doc_comment: Requirement,
    pub doc_comment_error_note: Option<String>,
    pub visibility: Requirement,
    pub visibility_error_note: Option<String>,
    pub decorators: HashMap<InternedString, DecoratorRule>,
}

impl AttributeRule {
    // TODO: we need std_rules based on AttributeKind
    //       for example, `trait` is only allowed for functions
    pub fn add_std_rules(&mut self, intermediate_dir: &str) {
        for (name, mut decorator) in [
            (
                "built_in",
                DecoratorRule {
                    requirement: Requirement::Maybe,
                    arg_requirement: Requirement::Never,
                    ..DecoratorRule::default()
                },
            ),
            (
                "any_type",
                DecoratorRule {
                    requirement: Requirement::Maybe,
                    arg_requirement: Requirement::Must,
                    arg_count: ArgCount::Gt(0),
                    arg_count_error_note: Some(String::from("Please give a list of generic parameters.")),
                    arg_type: ArgType::Generic,
                    arg_type_error_note: Some(String::from("It's used to turn off type-checking of generic parameters.")),
                    ..DecoratorRule::default()
                },
            ),
            (
                "lang_item",
                DecoratorRule {
                    requirement: Requirement::Maybe,
                    arg_requirement: Requirement::Must,
                    arg_count: ArgCount::Eq(1),
                    arg_count_error_note: Some(String::from("An item can have at most 1 lang item.")),
                    arg_type: ArgType::StringLiteral,
                    arg_type_error_note: Some(String::from("A lang item must be a string literal, which is compile-time-evaluable.")),
                    ..DecoratorRule::default()
                },
            ),
            (
                "lang_item_generics",
                DecoratorRule {
                    requirement: Requirement::Maybe,
                    arg_requirement: Requirement::Must,
                    arg_count: ArgCount::Gt(0),
                    arg_count_error_note: None,
                    arg_type: ArgType::StringLiteral,
                    arg_type_error_note: Some(String::from("A lang item must be a string literal, which is compile-time-evaluable.")),
                    ..DecoratorRule::default()
                },
            ),
        ] {
            let name = intern_string(name.as_bytes(), intermediate_dir).unwrap();
            decorator.name = name.clone();
            self.decorators.insert(name, decorator);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Requirement {
    Must,
    Maybe,
    Never,
}

#[derive(Clone, Debug)]
pub struct Visibility {
    pub keyword_span: Option<Span>,
    // TODO: more fields
}

impl Visibility {
    pub fn from_ast(ast_visibility: &ast::Visibility, session: &mut Session) -> Result<Visibility, ()> {
        Ok(Visibility {
            keyword_span: Some(ast_visibility.keyword_span),
            // TODO: more fields
        })
    }

    pub fn private() -> Visibility {
        Visibility {
            keyword_span: None,
        }
    }

    pub fn is_public(&self) -> bool {
        // TODO: more fine-grained visibility control
        self.keyword_span.is_some()
    }
}

#[derive(Clone, Debug)]
pub struct Decorator {
    pub name: InternedString,
    pub name_span: Span,
    pub args: Vec<Expr>,
    pub keyword_args: HashMap<InternedString, Expr>,
}

#[derive(Clone, Debug)]
pub struct DecoratorRule {
    pub name: InternedString,
    pub requirement: Requirement,

    // `ArgCount::Zero` and `Requirement::Never` are different.
    // `ArgCount::Zero` is `#[note()]`, while `Requirement::Never` is `#[note]`.
    pub arg_requirement: Requirement,
    pub arg_count: ArgCount,
    pub arg_count_error_note: Option<String>,
    pub arg_type: ArgType,
    pub arg_type_error_note: Option<String>,

    pub keyword_args: HashMap<InternedString, KeywordArgRule>,
}

impl Default for DecoratorRule {
    fn default() -> DecoratorRule {
        DecoratorRule {
            name: InternedString::empty(),
            requirement: Requirement::Never,
            arg_requirement: Requirement::Never,
            arg_count: ArgCount::Zero,
            arg_count_error_note: None,
            arg_type: ArgType::Expr,
            arg_type_error_note: None,
            keyword_args: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct KeywordArgRule {
    pub requirement: Requirement,
    pub requirement_error_note: Option<String>,
    pub arg_type: ArgType,
    pub arg_type_error_note: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub enum ArgType {
    Expr,
    StringLiteral,
    Generic,
    Path,
}

#[derive(Clone, Copy, Debug)]
pub enum ArgCount {
    Zero,
    Eq(usize),
    Gt(usize),
    Lt(usize),
}

fn check_arg_type(arg: &Expr, arg_type: ArgType, error_note: &Option<String>, session: &mut Session) -> Result<(), ()> {
    match (arg_type, arg) {
        (ArgType::Expr, _) => Ok(()),
        (ArgType::StringLiteral, Expr::String { .. }) => Ok(()),
        (ArgType::StringLiteral, _) => {
            session.errors.push(Error {
                // It's not a type error. An f-string token has type `String`, but it's still an error.
                kind: ErrorKind::UnexpectedToken {
                    expected: ErrorToken::String,
                    got: ErrorToken::Expr,
                },
                spans: arg.error_span().simple_error(),
                note: error_note.clone(),
            });
            Err(())
        },
        (ArgType::Generic, Expr::Identifier(IdentWithOrigin { origin: NameOrigin::Generic { .. }, .. })) => Ok(()),
        (ArgType::Generic, _) => {
            session.errors.push(Error {
                kind: ErrorKind::UnexpectedToken {
                    expected: ErrorToken::Generic,
                    got: ErrorToken::Expr,
                },
                spans: arg.error_span().simple_error(),
                note: error_note.clone(),
            });
            Err(())
        },
        (ArgType::Path, Expr::Identifier(_)) => Ok(()),
        (ArgType::Path, Expr::Path { lhs, .. }) if matches!(&**lhs, Expr::Identifier(_)) => Ok(()),
        (ArgType::Path, _) => {
            session.errors.push(Error {
                kind: ErrorKind::UnexpectedToken {
                    expected: ErrorToken::Path,
                    got: ErrorToken::Expr,
                },
                spans: arg.error_span().simple_error(),
                note: error_note.clone(),
            });
            Err(())
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AttributeRuleKey {
    pub kind: AttributeKind,
    pub is_std: bool,
    pub is_top_level: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AttributeKind {
    Alias,
    Assert,
    Enum,
    EnumVariant,
    Func,
    Let,
    Module,
    Struct,
    Use,
}
