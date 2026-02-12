use crate::{
    Attribute,
    AttributeRule,
    Requirement,
    Session,
    StructField,
    Type,
    Visibility,
    get_decorator_error_notes,
};
use sodigy_error::{Error, ErrorKind, ItemKind};
use sodigy_name_analysis::{Namespace, NameKind, UseCount};
use sodigy_parse::{self as ast, Generic};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

// TODO: attributes
#[derive(Clone, Debug)]
pub struct Enum {
    pub visibility: Visibility,
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<Generic>,
    pub variants: Vec<EnumVariant>,
}

// TODO: attributes
#[derive(Clone, Debug)]
pub struct EnumVariant {
    pub name: InternedString,
    pub name_span: Span,
    pub fields: EnumVariantFields,
}

// TODO: attributes
#[derive(Clone, Debug)]
pub enum EnumVariantFields {
    None,
    Tuple(Vec<Type>),
    Struct(Vec<StructField>),
}

impl Enum {
    pub fn from_ast(ast_enum: &ast::Enum, session: &mut Session) -> Result<Enum, ()> {
        let mut has_error = false;
        let mut variants = Vec::with_capacity(ast_enum.variants.as_ref().map(|variants| variants.len()).unwrap_or(0));

        let mut generic_params = HashMap::new();
        let mut generic_index = HashMap::new();

        for (index, generic) in ast_enum.generics.iter().enumerate() {
            generic_params.insert(generic.name, (generic.name_span, NameKind::GenericParam, UseCount::new()));
            generic_index.insert(generic.name, index);
        }

        session.name_stack.push(Namespace::GenericParam {
            names: generic_params,
            index: generic_index,
        });

        let attribute = match session.lower_attribute(
            &ast_enum.attribute,
            ItemKind::Enum,
            ast_enum.keyword_span,
        ) {
            Ok(attribute) => attribute,
            Err(()) => {
                has_error = true;
                Attribute::new()
            },
        };
        let visibility = attribute.visibility.clone();
        let built_in = attribute.get_decorator(b"built_in", &session.intermediate_dir).is_some();

        if let Err(()) = session.collect_lang_items(
            &attribute,
            ast_enum.name_span,
            Some(&ast_enum.generics),
            ast_enum.generic_group_span,
        ) {
            has_error = true;
        }

        if let Some(ast_variants) = &ast_enum.variants {
            for ast_variant in ast_variants.iter() {
                match EnumVariant::from_ast(ast_variant, session) {
                    Ok(variant) => {
                        variants.push(variant);
                    },
                    Err(()) => {
                        has_error = true;
                    },
                }
            }
        }

        else if !built_in {
            session.errors.push(Error {
                kind: ErrorKind::EnumWithoutBody,
                spans: ast_enum.name_span.simple_error(),
                note: None,
            });
            has_error = true;
        }

        let Some(Namespace::GenericParam { names, .. }) = session.name_stack.pop() else { unreachable!() };
        session.warn_unused_names(&names);

        if has_error {
            Err(())
        }

        else {
            Ok(Enum {
                visibility,
                keyword_span: ast_enum.keyword_span,
                name: ast_enum.name,
                name_span: ast_enum.name_span,
                generics: ast_enum.generics.clone(),
                variants,
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
            decorator_error_notes: get_decorator_error_notes(ItemKind::Enum, intermediate_dir),
        };

        if is_std {
            attribute_rule.add_decorators_for_std(ItemKind::Enum, intermediate_dir);
        }

        attribute_rule
    }
}

impl EnumVariant {
    pub fn from_ast(ast_variant: &ast::EnumVariant, session: &mut Session) -> Result<EnumVariant, ()> {
        let mut has_error = false;

        let attribute = match session.lower_attribute(
            &ast_variant.attribute,
            ItemKind::EnumVariant,

            // TODO: it has to be keyword_span, but a variant doesn't have a keyword_span!!
            ast_variant.name_span,
        ) {
            Ok(attribute) => attribute,
            Err(()) => {
                has_error = true;
                Attribute::new()
            },
        };

        if let Err(()) = session.collect_lang_items(
            &attribute,
            ast_variant.name_span,
            None,
            None,
        ) {
            has_error = true;
        }

        let fields = match &ast_variant.fields {
            ast::EnumVariantFields::None => EnumVariantFields::None,
            ast::EnumVariantFields::Tuple(ast_types) => {
                let mut types = Vec::with_capacity(ast_types.len());

                // TODO: attribute
                for (ast_type, _) in ast_types.iter() {
                    match Type::from_ast(ast_type, session) {
                        Ok(r#type) => {
                            types.push(r#type);
                        },
                        Err(()) => {
                            has_error = true;
                        },
                    }
                }

                EnumVariantFields::Tuple(types)
            },
            ast::EnumVariantFields::Struct(ast_fields) => {
                let mut fields = Vec::with_capacity(ast_fields.len());

                // TODO: attribute
                for ast_field in ast_fields.iter() {
                    match StructField::from_ast(ast_field, session) {
                        Ok(field) => {
                            fields.push(field);
                        },
                        Err(()) => {
                            has_error = true;
                        },
                    }
                }

                EnumVariantFields::Struct(fields)
            },
        };

        if has_error {
            Err(())
        }

        else {
            Ok(EnumVariant {
                name: ast_variant.name,
                name_span: ast_variant.name_span,
                fields,
            })
        }
    }

    pub fn get_attribute_rule(is_top_level: bool, is_std: bool, intermediate_dir: &str) -> AttributeRule {
        let mut attribute_rule = AttributeRule {
            doc_comment: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            doc_comment_error_note: Some(String::from("TODO: I'm not sure whether I should allow adding doc comments to inline items... maybe I have to do so?")),
            visibility: Requirement::Never,
            visibility_error_note: Some(String::from("You cannot set visibility of individual variants. If the enum is public, all the variants are public, and vice versa.")),
            decorators: HashMap::new(),
            decorator_error_notes: get_decorator_error_notes(ItemKind::EnumVariant, intermediate_dir),
        };

        if is_std {
            attribute_rule.add_decorators_for_std(ItemKind::EnumVariant, intermediate_dir);
        }

        attribute_rule
    }
}
