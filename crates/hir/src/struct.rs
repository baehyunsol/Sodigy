use crate::{
    Attribute,
    AttributeRule,
    Expr,
    FuncArgDef,
    GenericDef,
    Requirement,
    Session,
    Type,
    Visibility,
};
use sodigy_error::{Warning, WarningKind};
use sodigy_name_analysis::{Counter, Namespace, NameKind, UseCount};
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

pub struct Struct {
    pub visibility: Visibility,
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<GenericDef>,
    pub fields: Vec<StructField<Type>>,
}

pub type StructField<T> = FuncArgDef<T>;

#[derive(Clone, Debug)]
pub struct StructInitField {
    pub name: InternedString,
    pub name_span: Span,
    pub value: Expr,
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

        // TODO: I want it to be static
        let attribute_rule = AttributeRule {
            doc_comment: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            doc_comment_error_note: Some(String::from("You can only add doc comments to top-level items.")),
            visibility: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            visibility_error_note: Some(String::from("Only top-level items can be public.")),
            decorators: HashMap::new(),
        };

        let attribute = match Attribute::from_ast(&ast_struct.attribute, session, &attribute_rule, ast_struct.keyword_span) {
            Ok(attribute) => attribute,
            Err(()) => {
                has_error = true;
                Attribute::new()
            },
        };
        let visibility = attribute.visibility.clone();

        if let Some(lang_item) = attribute.lang_item(&session.intermediate_dir) {
            session.lang_items.insert(lang_item, ast_struct.name_span);
        }

        if let Some(lang_item_generics) = attribute.lang_item_generics(&session.intermediate_dir) {
            if lang_item_generics.len() == ast_struct.generics.len() {
                for i in 0..ast_struct.generics.len() {
                    session.lang_items.insert(lang_item_generics[i].to_string(), ast_struct.generics[i].name_span);
                }
            }

            else {
                // What kinda error should it throw?
                todo!()
            }
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
}
