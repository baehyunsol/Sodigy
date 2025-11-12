use crate::{
    Attribute,
    AttributeRule,
    Requirement,
    Session,
    StructFieldDef,
    Type,
    Visibility,
};
use sodigy_error::{Warning, WarningKind};
use sodigy_name_analysis::{Counter, Namespace, NameKind, UseCount};
use sodigy_parse::{self as ast, GenericDef};
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
    pub generics: Vec<GenericDef>,
    pub variants: Vec<EnumVariantDef>,
}

// TODO: attributes
#[derive(Clone, Debug)]
pub struct EnumVariantDef {
    pub name: InternedString,
    pub name_span: Span,
    pub args: EnumVariantArgs,
}

// TODO: attributes
#[derive(Clone, Debug)]
pub enum EnumVariantArgs {
    None,
    Tuple(Vec<Type>),
    Struct(Vec<StructFieldDef>),
}

impl Enum {
    pub fn from_ast(
        ast_enum: &ast::Enum,
        session: &mut Session,
        is_top_level: bool,
    ) -> Result<Enum, ()> {
        let mut has_error = false;
        let mut variants = Vec::with_capacity(ast_enum.variants.len());

        let mut generic_names = HashMap::new();
        let mut generic_index = HashMap::new();

        for (index, generic) in ast_enum.generics.iter().enumerate() {
            generic_names.insert(generic.name, (generic.name_span, NameKind::Generic, UseCount::new()));
            generic_index.insert(generic.name, index);
        }

        session.name_stack.push(Namespace::Generic {
            names: generic_names,
            index: generic_index,
        });

        // TODO: I want it to be static
        let mut attribute_rule = AttributeRule {
            doc_comment: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            doc_comment_error_note: Some(String::from("You can only add doc comments to top-level items.")),
            visibility: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            visibility_error_note: Some(String::from("Only top-level items can be public.")),
            decorators: HashMap::new(),
        };

        if session.is_std {
            attribute_rule.add_std_rules(&session.intermediate_dir);
        }

        let attribute = match Attribute::from_ast(&ast_enum.attribute, session, &attribute_rule, ast_enum.keyword_span) {
            Ok(attribute) => attribute,
            Err(()) => {
                has_error = true;
                Attribute::new()
            },
        };
        let visibility = attribute.visibility.clone();

        if let Some(lang_item) = attribute.lang_item(&session.intermediate_dir) {
            session.lang_items.insert(lang_item, ast_enum.name_span);
        }

        if let Some(lang_item_generics) = attribute.lang_item_generics(&session.intermediate_dir) {
            if lang_item_generics.len() == ast_enum.generics.len() {
                for i in 0..ast_enum.generics.len() {
                    session.lang_items.insert(lang_item_generics[i].to_string(), ast_enum.generics[i].name_span);
                }
            }

            else {
                // What kinda error should it throw?
                todo!()
            }
        }

        for ast_variant in ast_enum.variants.iter() {
            match EnumVariantDef::from_ast(ast_variant, session, is_top_level) {
                Ok(variant) => {
                    variants.push(variant);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        let Some(Namespace::Generic { names, .. }) = session.name_stack.pop() else { unreachable!() };

        for (name, (span, kind, count)) in names.iter() {
            if (!session.is_in_debug_context && count.always == Counter::Never) ||
                (session.is_in_debug_context && count.debug_only == Counter::Never) {
                let mut note = None;

                if count.debug_only != Counter::Never {
                    note = Some(String::from("This value is only used in debug mode."));
                }

                session.warnings.push(Warning {
                    kind: WarningKind::UnusedName {
                        name: *name,
                        kind: *kind,
                    },
                    spans: span.simple_error(),
                    note,
                });
            }
        }

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
}

impl EnumVariantDef {
    pub fn from_ast(
        ast_variant: &ast::EnumVariantDef,
        session: &mut Session,
        is_top_level: bool,
    ) -> Result<EnumVariantDef, ()> {
        let mut has_error = false;
        let args = match &ast_variant.args {
            ast::EnumVariantArgs::None => EnumVariantArgs::None,
            ast::EnumVariantArgs::Tuple(ast_types) => {
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

                EnumVariantArgs::Tuple(types)
            },
            ast::EnumVariantArgs::Struct(ast_fields) => {
                let mut fields = Vec::with_capacity(ast_fields.len());

                // TODO: attribute
                for ast_field in ast_fields.iter() {
                    match StructFieldDef::from_ast(ast_field, session, is_top_level) {
                        Ok(field) => {
                            fields.push(field);
                        },
                        Err(()) => {
                            has_error = true;
                        },
                    }
                }

                EnumVariantArgs::Struct(fields)
            },
        };

        if has_error {
            Err(())
        }

        else {
            Ok(EnumVariantDef {
                name: ast_variant.name,
                name_span: ast_variant.name_span,
                args,
            })
        }
    }
}
