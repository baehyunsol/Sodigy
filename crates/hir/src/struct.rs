use crate::{
    AssociatedFunc,
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
use sodigy_error::{Error, ErrorKind, ItemKind, Lint, LintKind, comma_list_strs};
use sodigy_name_analysis::{Namespace, NameKind, UseCount};
use sodigy_parse as ast;
use sodigy_span::{RenderableSpan, Span};
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
    pub generic_group_span: Option<Span>,
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

// `crates/hir/src/lib.rs` will tell you what's the difference between Struct vs StructShape
#[derive(Clone, Debug)]
pub struct StructShape {
    // If it's a variant of an enum, this field has the def_span of the enum.
    pub from_enum: Option<Span>,

    pub name: InternedString,
    pub fields: Vec<StructField>,
    pub generics: Vec<Generic>,
    pub generic_group_span: Option<Span>,
    pub associated_funcs: HashMap<InternedString, AssociatedFunc>,
    pub associated_lets: HashMap<InternedString, Span>,
}

impl Struct {
    pub fn from_ast(ast_struct: &ast::Struct, session: &mut Session) -> Result<Struct, ()> {
        let mut has_error = false;
        let mut fields = Vec::with_capacity(ast_struct.fields.as_ref().map(|fields| fields.len()).unwrap_or(0));

        let mut generic_params = HashMap::new();
        let mut generic_index = HashMap::new();

        for (index, generic) in ast_struct.generics.iter().enumerate() {
            generic_params.insert(generic.name, (generic.name_span.clone(), NameKind::GenericParam, UseCount::new()));
            generic_index.insert(generic.name, index);
            session.generic_to_def_span.insert(generic.name_span.clone(), ast_struct.name_span.clone());
        }

        session.name_stack.push(Namespace::GenericParam {
            names: generic_params,
            index: generic_index,
        });

        let attribute = match session.lower_attribute(
            &ast_struct.attribute,
            ItemKind::Struct,
            ast_struct.keyword_span.clone(),
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
            ast_struct.name_span.clone(),
            Some(&ast_struct.generics),
            ast_struct.generic_group_span.clone(),
        ) {
            has_error = true;
        }

        let mut missing_type_annots = vec![];
        let mut wildcard_spans = vec![];

        if let Some(ast_fields) = &ast_struct.fields {
            for ast_field in ast_fields.iter() {
                match StructField::from_ast(ast_field, session) {
                    Ok(field) => {
                        match &field.type_annot {
                            Some(r#type) => {
                                let w = r#type.get_wildcard_spans();

                                if !w.is_empty() {
                                    missing_type_annots.push(field.name);
                                    wildcard_spans.extend(w);
                                }
                            },
                            None => {
                                missing_type_annots.push(field.name);
                            },
                        }

                        fields.push(field);
                    },
                    Err(()) => {
                        has_error = true;
                    },
                }
            }
        }

        else if !built_in {
            session.errors.push(Error {
                kind: ErrorKind::StructWithoutBody,
                spans: ast_struct.name_span.simple_error(),
                note: None,
            });
            has_error = true;
        }

        if !missing_type_annots.is_empty() {
            let help_message = format!(
                "Type annotation{} for the field{} {} {} missing.",
                if missing_type_annots.len() == 1 { "" } else { "s" },
                if missing_type_annots.len() == 1 { "" } else { "s" },
                comma_list_strs(
                    &missing_type_annots.iter().map(|name| name.unintern_or_default(&session.intermediate_dir)).collect::<Vec<_>>(),
                    "`",
                    "`",
                    "and",
                ),
                if missing_type_annots.len() == 1 { "is" } else { "are" },
            );
            let mut error_spans = ast_struct.name_span.simple_error();
            error_spans.extend(wildcard_spans.drain(..).map(
                |span| RenderableSpan {
                    span,
                    auxiliary: true,
                    note: Some(String::from("This is an incomplete type annotation.")),
                }
            ));

            if ast_struct.generics.is_empty() {
                session.warnings.push(Lint {
                    kind: LintKind::StructWithoutTypeAnnot,
                    spans: error_spans,
                    note: Some(help_message),
                });
            } else {
                has_error = true;
                session.errors.push(Error {
                    kind: ErrorKind::GenericStructWithoutTypeAnnot,
                    spans: error_spans,
                    note: Some(format!("A generic struct needs type annotations because the compiler cannot infer the types otherwise.\n{help_message}")),
                });
            }
        }

        let Some(Namespace::GenericParam { names, .. }) = session.name_stack.pop() else { unreachable!() };

        if !built_in {
            session.warn_unused_names(&names);
        }

        if has_error {
            Err(())
        }

        else {
            Ok(Struct {
                visibility,
                keyword_span: ast_struct.keyword_span.clone(),
                name: ast_struct.name,
                name_span: ast_struct.name_span.clone(),
                generics: ast_struct.generics.clone(),
                generic_group_span: ast_struct.generic_group_span.clone(),
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

    pub fn shape(&self) -> StructShape {
        StructShape {
            from_enum: None,
            name: self.name,

            // It's not gonna use this type annotation anymore.
            // It'll use `types` in the mir-session or mir-global-context.
            // Let's save some space by removing the type info.
            fields: remove_struct_fields_type_annot(&self.fields),

            generics: self.generics.clone(),
            generic_group_span: self.generic_group_span.clone(),
            associated_funcs: HashMap::new(),
            associated_lets: HashMap::new(),
        }
    }
}

pub fn remove_struct_fields_type_annot(fields: &[StructField]) -> Vec<StructField> {
    fields.iter().map(
        |field| StructField {
            name: field.name,
            name_span: field.name_span.clone(),
            default_value: field.default_value.clone(),
            type_annot: None,
        }
    ).collect()
}
