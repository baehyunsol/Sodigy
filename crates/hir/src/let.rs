use crate::{
    ArgCount,
    ArgType,
    AssociatedItem,
    Attribute,
    AttributeRule,
    DecoratorRule,
    Expr,
    Requirement,
    Session,
    Type,
    TypeAssertion,
    Visibility,
    get_decorator_error_notes,
};
use sodigy_error::ItemKind;
use sodigy_name_analysis::{NameOrigin, Namespace};
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Let {
    pub visibility: Visibility,
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub type_annot: Option<Type>,
    pub value: Expr,
    pub origin: LetOrigin,

    // We have to do cycle checks.
    pub foreign_names: HashMap<InternedString, (NameOrigin, Span /* def_span */)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LetOrigin {
    TopLevel,
    Inline,  // `let` keyword in an inline block

    // TODO: distinguish struct default values and func default values
    FuncDefaultValue,

    // `match` expressions are lowered to blocks
    Match,
}

impl Let {
    pub fn from_ast(
        ast_let: &ast::Let,
        session: &mut Session,
        is_top_level: bool,
    ) -> Result<Let, ()> {
        let mut has_error = false;
        let mut type_annot = None;

        let attribute = match session.lower_attribute(
            &ast_let.attribute,
            ItemKind::Let,
            ast_let.keyword_span,
            is_top_level,
        ) {
            Ok(attribute) => attribute,
            Err(()) => {
                has_error = true;
                Attribute::new()
            },
        };
        let visibility = attribute.visibility.clone();

        if let Some(asserted_type) = attribute.get_decorator(b"assert_type", &session.intermediate_dir) {
            session.type_assertions.push(TypeAssertion {
                name_span: ast_let.name_span,
                type_span: asserted_type.args[0].error_span_wide(),
                r#type: asserted_type.args[0].clone().unwrap_type(),
            });
        }

        if let Some(association) = attribute.get_decorator(b"associate", &session.intermediate_dir) {
            session.associated_items.push(AssociatedItem {
                is_func: false,
                name: ast_let.name,
                name_span: ast_let.name_span,
                params: None,
                type_span: association.args[0].error_span_wide(),
                r#type: association.args[0].clone().unwrap_type(),
            });
        }

        if let Some(ast_type) = &ast_let.type_annot {
            match Type::from_ast(ast_type, session) {
                Ok(ty) => {
                    type_annot = Some(ty);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        session.name_stack.push(Namespace::ForeignNameCollector {
            is_func: false,
            foreign_names: HashMap::new(),
        });

        let value = match Expr::from_ast(&ast_let.value, session) {
            Ok(value) => Some(value),
            Err(()) => {
                has_error = true;
                None
            },
        };

        let Some(Namespace::ForeignNameCollector { foreign_names, .. }) = session.name_stack.pop() else { unreachable!() };

        if has_error {
            Err(())
        }

        else {
            Ok(Let {
                visibility,
                keyword_span: ast_let.keyword_span,
                name: ast_let.name,
                name_span: ast_let.name_span,
                type_annot,
                value: value.unwrap(),
                origin: if is_top_level {
                    LetOrigin::TopLevel
                } else {
                    LetOrigin::Inline
                },
                foreign_names,
            })
        }
    }

    pub fn get_attribute_rule(is_top_level: bool, is_std: bool, intermediate_dir: &str) -> AttributeRule {
        let mut attribute_rule = AttributeRule {
            doc_comment: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            doc_comment_error_note: Some(String::from("You can only add doc comments to top-level items.")),
            visibility: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            visibility_error_note: Some(String::from("Only top-level items can be public.")),
            decorators: vec![
                (
                    intern_string(b"assert_type", intermediate_dir).unwrap(),
                    DecoratorRule {
                        name: intern_string(b"assert_type", intermediate_dir).unwrap(),
                        requirement: Requirement::Maybe,
                        arg_requirement: Requirement::Must,
                        arg_count: ArgCount::Eq(1),
                        arg_type: ArgType::Type,
                        arg_type_error_note: Some(String::from("Please give me the type of the value.")),
                        ..DecoratorRule::default()
                    },
                ), (
                    intern_string(b"associate", intermediate_dir).unwrap(),
                    DecoratorRule {
                        name: intern_string(b"associate", intermediate_dir).unwrap(),
                        requirement: Requirement::Maybe,
                        arg_requirement: Requirement::Must,
                        arg_count: ArgCount::Eq(1),
                        arg_count_error_note: Some(String::from("You can associate at most 1 type with a value.")),
                        arg_type: ArgType::Type,
                        arg_type_error_note: Some(String::from("The argument must be a type that you want to associate the value with.")),
                        ..DecoratorRule::default()
                    },
                ),
            ].into_iter().collect(),
            decorator_error_notes: get_decorator_error_notes(ItemKind::Let, intermediate_dir),
        };

        if is_std {
            attribute_rule.add_decorators_for_std(ItemKind::Let, intermediate_dir);
        }

        attribute_rule
    }
}
