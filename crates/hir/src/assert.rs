use crate::{
    ArgCount,
    ArgType,
    Attribute,
    AttributeRule,
    DecoratorArg,
    DecoratorRule,
    Expr,
    Requirement,
    Session,
    get_decorator_error_notes,
};
use sodigy_error::ItemKind;
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};
use std::collections::hash_map::HashMap;

#[derive(Clone, Debug)]
pub struct Assert {
    // A name of an assertion must be a string literal, but you can use
    // any string expression as a note.
    // e.g. `#[name("test1")]` is valid,
    //      `#[name(f"test{i}")]` is not valid,
    //      `#[name(test1)]` is not valid,
    //      `#[note("It is a test")]` is valid,
    //      `#[note(f"check {a}+{b}={a+b}")]` is valid,
    //      `#[note(3 + 4)]` is not valid (type error).
    // I chose this way because
    //
    // 1. In order to create a test harness, it has to be easy for the compiler
    //    to know the name of the assertions. So, I don't want a runtime-evaluation.
    // 2. If it uses an identifier instead of a string literal, there are much less
    //    characters to use. For example, the user might want to use colons in the
    //    name of an assertion.
    // 3. `#[note]` must be very flexible.
    pub name: Option<InternedString>,
    pub note: Option<Expr>,
    pub note_decorator_span: Option<Span>,

    pub keyword_span: Span,
    pub value: Expr,

    // By default, assertions are enabled only in debug profile.
    // If it has `#[always]` decorator, it's always enabled.
    pub always: bool,
}

impl Assert {
    pub fn from_ast(ast_assert: &ast::Assert, session: &mut Session) -> Result<Assert, ()> {
        let mut has_error = false;

        let attribute = match session.lower_attribute(
            &ast_assert.attribute,
            ItemKind::Assert,
            ast_assert.keyword_span,
        ) {
            Ok(attribute) => AssertAttribute::from_attribute(&attribute, session),
            Err(()) => {
                has_error = true;
                AssertAttribute::default()
            },
        };

        let is_in_debug_context_prev = session.is_in_debug_context;
        session.is_in_debug_context = !attribute.always;

        let value = match Expr::from_ast(&ast_assert.value, session) {
            Ok(value) => Some(value),
            Err(()) => {
                has_error = true;
                None
            },
        };

        session.is_in_debug_context = is_in_debug_context_prev;

        if has_error {
            Err(())
        }

        else {
            Ok(Assert {
                name: attribute.name,
                note: attribute.note,
                note_decorator_span: attribute.note_decorator_span,
                keyword_span: ast_assert.keyword_span,
                value: value.unwrap(),
                always: attribute.always,
            })
        }
    }

    pub fn get_attribute_rule(is_top_level: bool, is_std: bool, intermediate_dir: &str) -> AttributeRule {
        let mut decorators = vec![
            (
                intern_string(b"name", intermediate_dir).unwrap(),
                DecoratorRule {
                    name: intern_string(b"name", intermediate_dir).unwrap(),
                    requirement: Requirement::Maybe,
                    arg_requirement: Requirement::Must,
                    arg_count: ArgCount::Eq(1),
                    arg_count_error_note: Some(String::from("A name of an assertion must be unique.")),
                    arg_type: ArgType::StringLiteral,
                    arg_type_error_note: Some(String::from("A name of an assertion must be a string literal, which is compile-time-evaluable.")),
                    keyword_args: HashMap::new(),
                },
            ),
            (
                intern_string(b"note", intermediate_dir).unwrap(),
                DecoratorRule {
                    name: intern_string(b"note", intermediate_dir).unwrap(),
                    requirement: Requirement::Maybe,
                    arg_requirement: Requirement::Must,
                    arg_count: ArgCount::Eq(1),
                    arg_count_error_note: Some(String::from("There must be at most 1 note for an assertion.")),
                    arg_type: ArgType::Expr,
                    arg_type_error_note: None,  // infallible
                    keyword_args: HashMap::new(),
                },
            ),
        ];

        if !is_top_level {
            decorators.push((
                intern_string(b"always", intermediate_dir).unwrap(),
                DecoratorRule {
                    name: intern_string(b"always", intermediate_dir).unwrap(),
                    requirement: Requirement::Maybe,
                    arg_requirement: Requirement::Never,
                    ..DecoratorRule::default()
                },
            ));
        }

        let mut attribute_rule = AttributeRule {
            doc_comment: Requirement::Never,
            doc_comment_error_note: Some(String::from("Use `#[note(\"...\")]` decorator instead.")),
            visibility: Requirement::Never,
            visibility_error_note: Some(String::from("You cannot set visibility of an assertion.")),
            decorators: decorators.into_iter().collect(),
            decorator_error_notes: get_decorator_error_notes(ItemKind::Assert, intermediate_dir),
        };

        if is_std {
            attribute_rule.add_decorators_for_std(ItemKind::Assert, intermediate_dir);
        }

        attribute_rule
    }
}

#[derive(Clone, Debug)]
pub struct AssertAttribute {
    pub name: Option<InternedString>,
    pub note: Option<Expr>,
    pub note_decorator_span: Option<Span>,
    pub always: bool,
}

impl Default for AssertAttribute {
    fn default() -> Self {
        AssertAttribute {
            name: None,
            note: None,
            note_decorator_span: None,
            always: false,
        }
    }
}

impl AssertAttribute {
    // It never fails because `Attribute::from_ast` does all the checks.
    pub fn from_attribute(
        attribute: &Attribute,
        session: &mut Session,
    ) -> AssertAttribute {
        let mut name = None;
        let mut note = None;
        let mut note_decorator_span = None;
        let mut always = false;

        if let Some(name_) = attribute.decorators.get(&intern_string(b"name", &session.intermediate_dir).unwrap()) {
            match name_.args.get(0) {
                Some(DecoratorArg::Expr(Expr::String { s, .. })) => {
                    name = Some(*s);
                },
                _ => unreachable!(),
            }
        }

        if let Some(note_) = attribute.decorators.get(&intern_string(b"note", &session.intermediate_dir).unwrap()) {
            match note_.args.get(0) {
                Some(e) => {
                    note_decorator_span = Some(note_.name_span);
                    note = Some(e.clone().unwrap_expr());
                },
                _ => unreachable!(),
            }
        }

        if attribute.decorators.contains_key(&intern_string(b"always", &session.intermediate_dir).unwrap()) {
            always = true;
        }

        AssertAttribute { name, note, note_decorator_span, always }
    }
}
