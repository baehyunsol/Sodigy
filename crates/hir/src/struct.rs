use crate::{
    Attribute,
    AttributeRule,
    Expr,
    FuncParam,
    Generic,
    Requirement,
    Session,
    Visibility,
    get_decorator_error_notes,
};
use sodigy_error::ItemKind;
use sodigy_name_analysis::{Namespace, NameKind, UseCount};
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

// TODO: attributes
#[derive(Clone, Debug)]
pub struct Struct {
    pub visibility: Visibility,
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<Generic>,
    pub fields: Vec<StructField>,
}

// TODO: attributes
pub type StructField = FuncParam;

#[derive(Clone, Debug)]
pub struct StructInitField {
    pub name: InternedString,
    pub name_span: Span,
    pub value: Expr,
}

#[derive(Clone, Debug)]
pub struct StructShape {
    pub name: InternedString,
    pub fields: Vec<StructField>,
    pub generics: Vec<Generic>,
}

impl Struct {
    pub fn from_ast(
        ast_struct: &ast::Struct,
        session: &mut Session,
        is_top_level: bool,
    ) -> Result<Struct, ()> {
        let mut has_error = false;
        let mut fields = Vec::with_capacity(ast_struct.fields.len());

        let mut generic_names = HashMap::new();
        let mut generic_index = HashMap::new();

        for (index, generic) in ast_struct.generics.iter().enumerate() {
            generic_names.insert(generic.name, (generic.name_span, NameKind::Generic, UseCount::new()));
            generic_index.insert(generic.name, index);
        }

        session.name_stack.push(Namespace::Generic {
            names: generic_names,
            index: generic_index,
        });

        let attribute = match session.lower_attribute(
            &ast_struct.attribute,
            ItemKind::Struct,
            ast_struct.keyword_span,
            is_top_level,
        ) {
            Ok(attribute) => attribute,
            Err(()) => {
                has_error = true;
                Attribute::new()
            },
        };
        let visibility = attribute.visibility.clone();

        if let Err(()) = session.collect_lang_items(
            &attribute,
            ast_struct.name_span,
            Some(&ast_struct.generics),
            ast_struct.generic_group_span,
        ) {
            has_error = true;
        }

        for field in ast_struct.fields.iter() {
            match StructField::from_ast(field, session, is_top_level) {
                Ok(field) => {
                    fields.push(field);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        let Some(Namespace::Generic { names, .. }) = session.name_stack.pop() else { unreachable!() };
        session.warn_unused_names(&names);

        if has_error {
            Err(())
        }

        else {
            Ok(Struct {
                visibility,
                keyword_span: ast_struct.keyword_span,
                name: ast_struct.name,
                name_span: ast_struct.name_span,
                generics: ast_struct.generics.clone(),
                fields,
            })
        }
    }

    pub fn get_attribute_rule(is_top_level: bool, is_std: bool, intermediate_dir: &str) -> AttributeRule {
        let mut attribute_rule = AttributeRule {
            doc_comment: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            doc_comment_error_note: Some(String::from("You can only add doc comments to top-level items.")),
            visibility: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            visibility_error_note: Some(String::from("Only top-level items can be public.")),
            decorators: HashMap::new(),
            decorator_error_notes: get_decorator_error_notes(ItemKind::Struct, intermediate_dir),
        };

        if is_std {
            attribute_rule.add_decorators_for_std(ItemKind::Struct, intermediate_dir);
        }

        attribute_rule
    }
}
