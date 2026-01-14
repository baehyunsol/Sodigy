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
    Type,
    Use,
};
use sodigy_error::{Error, ErrorKind, ErrorToken, ItemKind, comma_list_strs};
use sodigy_name_analysis::{IdentWithOrigin, NameOrigin};
use sodigy_parse::{self as ast, DocComment};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::{InternedString, intern_string};
use std::collections::hash_map::{Entry, HashMap};

mod decorator_docs;

pub use decorator_docs::generate_decorator_docs;

impl Session {
    pub fn lower_attribute(
        &mut self,
        ast_attribute: &ast::Attribute,
        item: ItemKind,
        keyword_span: Span,
        is_top_level: bool,
    ) -> Result<Attribute, ()> {
        let attribute_rule_key = AttributeRuleKey {
            item,
            is_top_level,
            is_std: self.is_std,
        };
        let attribute_rule = match self.attribute_rule_cache.get(&attribute_rule_key) {
            Some(rule) => rule.clone(),
            None => {
                let rule = match item {
                    ItemKind::Alias => Alias::get_attribute_rule(is_top_level, self.is_std, &self.intermediate_dir),
                    ItemKind::Assert => Assert::get_attribute_rule(is_top_level, self.is_std, &self.intermediate_dir),
                    ItemKind::Enum => Enum::get_attribute_rule(is_top_level, self.is_std, &self.intermediate_dir),
                    ItemKind::EnumVariant => EnumVariant::get_attribute_rule(is_top_level, self.is_std, &self.intermediate_dir),
                    ItemKind::Func => Func::get_attribute_rule(is_top_level, self.is_std, &self.intermediate_dir),
                    ItemKind::Let => Let::get_attribute_rule(is_top_level, self.is_std, &self.intermediate_dir),
                    ItemKind::Module => Module::get_attribute_rule(is_top_level, self.is_std, &self.intermediate_dir),
                    ItemKind::Struct => Struct::get_attribute_rule(is_top_level, self.is_std, &self.intermediate_dir),
                    ItemKind::Use => Use::get_attribute_rule(is_top_level, self.is_std, &self.intermediate_dir),
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
        generic_group_span: Option<Span>,
    ) -> Result<(), ()> {
        if let Some(lang_item) = attribute.lang_item(&self.intermediate_dir) {
            self.lang_items.insert(lang_item, lang_item_span);
        }

        if let Some((deco_span, lang_item_generics)) = attribute.lang_item_generics(&self.intermediate_dir) {
            if let (Some(generic_defs), Some(generic_group_span)) = (generic_defs, generic_group_span) {
                if lang_item_generics.len() == generic_defs.len() {
                    for i in 0..generic_defs.len() {
                        self.lang_items.insert(lang_item_generics[i].to_string(), generic_defs[i].name_span);
                    }
                }

                else {
                    self.errors.push(Error {
                        kind: ErrorKind::WrongNumberOfLangItemGenerics {
                            lang_items: lang_item_generics.len(),
                            generic_def: generic_defs.len(),
                        },
                        spans: vec![
                            RenderableSpan {
                                span: deco_span,
                                auxiliary: false,
                                note: Some(format!(
                                    "#[lang_item_generics] has {} argument{}.",
                                    lang_item_generics.len(),
                                    if lang_item_generics.len() == 1 { "" } else { "s" },
                                )),
                            },
                            RenderableSpan {
                                span: generic_group_span,
                                auxiliary: true,
                                note: Some(format!(
                                    "It has {} generic parameter{}.",
                                    generic_defs.len(),
                                    if generic_defs.len() == 1 { "" } else { "s" },
                                )),
                            },
                        ],
                        note: None,
                    });
                    return Err(());
                }
            }

            else {
                self.errors.push(Error {
                    kind: ErrorKind::WrongNumberOfLangItemGenerics {
                        generic_def: 0,
                        lang_items: lang_item_generics.len(),
                    },
                    spans: vec![
                        RenderableSpan {
                            span: deco_span,
                            auxiliary: false,
                            note: Some(format!(
                                "#[lang_item_generics] has {} argument{}.",
                                lang_item_generics.len(),
                                if lang_item_generics.len() == 1 { "" } else { "s" },
                            )),
                        },
                        RenderableSpan {
                            span: lang_item_span,
                            auxiliary: true,
                            note: Some(String::from("There's no generic parameter.")),
                        },
                    ],
                    note: None,
                });
                return Err(());
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
                            let mut keyword_args: HashMap<InternedString, DecoratorArg> = HashMap::new();
                            let mut positional_args: Vec<&ast::DecoratorArg> = vec![];
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

                                            match DecoratorArg::from_ast(&ast_arg, *arg_type, session) {
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
                                        positional_args.push(ast_arg);
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
                                            span: arg.error_span_wide(),
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
                                            span: arg.error_span_wide(),
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
                                            span: arg.error_span_wide(),
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
                                        match DecoratorArg::from_ast(ast_arg, rule.arg_type, session) {
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
                    has_error = true;
                    session.errors.push(Error {
                        kind: ErrorKind::InvalidDecorator(ast_decorator.name),
                        spans: ast_decorator.name_span.simple_error(),
                        note: rule.decorator_error_notes.get(&ast_decorator.name).map(|n| n.to_string()),
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
                Some(DecoratorArg::Expr(Expr::String { s, .. })) => Some(s.unintern_or_default(intermediate_dir)),
                _ => unreachable!(),
            },
            None => None,
        }
    }

    pub fn lang_item_generics(&self, intermediate_dir: &str) -> Option<(Span, Vec<String>)> {
        match self.decorators.get(&intern_string(b"lang_item_generics", intermediate_dir).unwrap()) {
            Some(d) => Some((
                d.name_span,
                d.args.iter().map(
                    |arg| match arg {
                        DecoratorArg::Expr(Expr::String { s, .. }) => s.unintern_or_default(intermediate_dir),
                        _ => unreachable!(),
                    }
                ).collect(),
            )),
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

    // If the same key is both at `decorators` and `decorator_error_notes`, the entry at
    // `decorator_error_notes` is ignored. This is intentional: it makes making `decorator_error_notes` easier.
    pub decorator_error_notes: HashMap<InternedString, String>,
}

impl AttributeRule {
    pub fn add_decorators_for_std(&mut self, item_kind: ItemKind, intermediate_dir: &str) {
        for (name, kinds, mut decorator) in [
            (
                "built_in",
                &[ItemKind::Enum, ItemKind::Func, ItemKind::Struct][..],
                DecoratorRule {
                    requirement: Requirement::Maybe,
                    arg_requirement: Requirement::Never,
                    ..DecoratorRule::default()
                },
            ),
            (
                "lang_item",
                &[
                    ItemKind::Alias,
                    ItemKind::Enum,
                    ItemKind::EnumVariant,
                    ItemKind::Func,
                    ItemKind::Let,
                    ItemKind::Struct,
                ],
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
                &[
                    ItemKind::Alias,
                    ItemKind::Enum,
                    ItemKind::Func,
                    ItemKind::Struct,
                ],
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
            let name_interned = intern_string(name.as_bytes(), intermediate_dir).unwrap();
            decorator.name = name_interned.clone();

            if !kinds.contains(&item_kind) {
                self.decorator_error_notes.insert(
                    name_interned,
                    format!(
                        "Decorator `{name}` is allowed for {}, but not for {}.",
                        comma_list_strs(
                            &kinds.iter().map(|kind| kind.render().to_string()).collect::<Vec<_>>(),
                            "",
                            "",
                            "or",
                        ),
                        item_kind.render(),
                    ),
                );
                continue;
            }

            self.decorators.insert(name_interned, decorator);
        }
    }
}

// TODO: Am I over-engineering??
//       When I first drafted this, I thought it would be useful, but there isn't much decorators now.
//
// When the compiler throws `InvalidDecorator`, the compiler looks at this map to generate error note.
pub fn get_decorator_error_notes(item_kind: ItemKind, intermediate_dir: &str) -> HashMap<InternedString, String> {
    let mut result = vec![];

    for decorator in [
        "built_in",
        "lang_item",
        "lang_item_generics",
        "always",
    ] {
        let note = match (decorator, item_kind) {
            ("built_in", _) => "You cannot define a built-in item.",
            ("lang_item" | "lang_item_generics", _) => "Lang-items are only allowed for special items in std.",
            ("always", ItemKind::Assert) => "Only inline assertions can have this attribute.",
            _ => continue,
        };

        result.push((decorator, note));
    }

    result.iter().map(
        |(decorator, note)| (
            intern_string(decorator.as_bytes(), intermediate_dir).unwrap(),
            note.to_string(),
        )
    ).collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    pub args: Vec<DecoratorArg>,
    pub keyword_args: HashMap<InternedString, DecoratorArg>,
}

#[derive(Clone, Debug)]
pub enum DecoratorArg {
    Expr(Expr),
    Type(Type),
}

impl DecoratorArg {
    pub fn from_ast(ast_arg: &ast::DecoratorArg, arg_type: ArgType, session: &mut Session) -> Result<DecoratorArg, ()> {
        if arg_type.is_expr() {
            match &ast_arg.expr {
                Ok(expr) => Expr::from_ast(expr, session).map(|expr| DecoratorArg::Expr(expr)),
                Err(e) => {
                    session.errors.extend(e.clone());
                    Err(())
                },
            }
        }

        else {
            match &ast_arg.r#type {
                Ok(r#type) => Type::from_ast(r#type, session).map(|r#type| DecoratorArg::Type(r#type)),
                Err(e) => {
                    session.errors.extend(e.clone());
                    Err(())
                },
            }
        }
    }

    pub fn unwrap_expr(self) -> Expr {
        match self {
            DecoratorArg::Expr(expr) => expr,
            _ => panic!(),
        }
    }

    pub fn unwrap_type(self) -> Type {
        match self {
            DecoratorArg::Type(r#type) => r#type,
            _ => panic!(),
        }
    }

    pub fn error_span_narrow(&self) -> Span {
        match self {
            DecoratorArg::Expr(expr) => expr.error_span_narrow(),
            DecoratorArg::Type(r#type) => r#type.error_span_narrow(),
        }
    }

    pub fn error_span_wide(&self) -> Span {
        match self {
            DecoratorArg::Expr(expr) => expr.error_span_wide(),
            DecoratorArg::Type(r#type) => r#type.error_span_wide(),
        }
    }
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
    // These are all expressions
    Expr,
    StringLiteral,
    Path,

    // These are all type annotations
    Type,
    Generic,
}

impl ArgType {
    pub fn is_expr(&self) -> bool {
        matches!(self, ArgType::Expr | ArgType::StringLiteral | ArgType::Path)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ArgCount {
    Zero,
    Eq(usize),
    Gt(usize),
    Lt(usize),
}

fn check_arg_type(arg: &DecoratorArg, arg_type: ArgType, error_note: &Option<String>, session: &mut Session) -> Result<(), ()> {
    match (arg_type, arg) {
        (ArgType::Expr, DecoratorArg::Expr(_)) => Ok(()),
        (ArgType::Expr, DecoratorArg::Type(_)) => unreachable!(),
        (ArgType::StringLiteral, DecoratorArg::Expr(Expr::String { .. })) => Ok(()),
        (ArgType::StringLiteral, _) => {
            session.errors.push(Error {
                // It's not a type error. An f-string token has type `String`, but it's still an error.
                kind: ErrorKind::UnexpectedToken {
                    expected: ErrorToken::String,
                    got: ErrorToken::Expr,
                },
                spans: arg.error_span_wide().simple_error(),
                note: error_note.clone(),
            });
            Err(())
        },
        (ArgType::Path, DecoratorArg::Expr(Expr::Ident(_))) => Ok(()),
        (ArgType::Path, DecoratorArg::Expr(Expr::Path { lhs, .. })) if matches!(&**lhs, Expr::Ident(_)) => Ok(()),
        (ArgType::Path, _) => {
            session.errors.push(Error {
                kind: ErrorKind::UnexpectedToken {
                    expected: ErrorToken::Path,
                    got: ErrorToken::Expr,
                },
                spans: arg.error_span_wide().simple_error(),
                note: error_note.clone(),
            });
            Err(())
        },
        (ArgType::Type, DecoratorArg::Type(_)) => Ok(()),
        (ArgType::Type, DecoratorArg::Expr(_)) => unreachable!(),
        (ArgType::Generic, DecoratorArg::Type(Type::Ident(IdentWithOrigin { origin: NameOrigin::Generic { .. }, .. }))) => Ok(()),
        (ArgType::Generic, _) => {
            session.errors.push(Error {
                kind: ErrorKind::UnexpectedToken {
                    expected: ErrorToken::Generic,
                    got: ErrorToken::TypeAnnotation,
                },
                spans: arg.error_span_wide().simple_error(),
                note: error_note.clone(),
            });
            Err(())
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AttributeRuleKey {
    pub item: ItemKind,
    pub is_std: bool,
    pub is_top_level: bool,
}
