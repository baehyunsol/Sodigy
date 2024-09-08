use crate::enum_::StructVariantInfo;
use crate::func::{FuncKind, lower_ast_func};
use crate::names::{IdentWithOrigin, NameSpace};
use crate::session::HirSession;
use sodigy_ast::{self as ast, IdentWithSpan};
use sodigy_intern::{InternedString, InternSession};
use sodigy_lang_item::LangItem;
use sodigy_number::SodigyNumber;
use sodigy_session::SodigySession;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;
use std::collections::{HashMap, HashSet};

/*
let struct Message<T> = { data: T, id: Int };
->
let @@struct_constructor_Message<T>(data: T, id: Int): Message(T) = @@struct_body(data, id);
let Message<T>: Type = @@dummy;

`Message { data: "", id: 0 }` is lowered to `@@struct_constructor_Message("", 0)`.
`Message(String)`, which is a type annotation, is lowered to `Message<T>`.
*/
pub fn lower_ast_struct(
    name: &IdentWithSpan,
    generics: &Vec<ast::GenericDef>,
    fields: &Vec<ast::FieldDef>,
    uid: Uid,
    session: &mut HirSession,
    used_names: &mut HashSet<IdentWithOrigin>,
    imports: &HashMap<InternedString, (SpanRange, Vec<IdentWithSpan>)>,
    attributes: &Vec<ast::Attribute>,
    name_space: &mut NameSpace,
    variant_info: Option<StructVariantInfo>,
) -> Result<(), ()> {
    let type_name = match &variant_info {
        Some(StructVariantInfo { parent_name, .. }) => parent_name,
        None => name,
    };

    let constructor_name = IdentWithSpan::new(
        session.add_prefix(name.id(), "@@struct_constructor_"),
        *name.span(),
    );
    let mut constructor_body = create_struct_body(
        fields_to_values(fields, session.get_interner()),
        name.span().into_fake(),
        session.get_interner(),
    );

    if let Some(StructVariantInfo { variant_index, .. }) = variant_info {
        constructor_body = wrap_struct_body_with_enum_body(
            constructor_body,
            variant_index,
            session.get_interner(),
        );
    }

    let constructor = lower_ast_func(
        &constructor_name,
        generics,
        Some(&fields_to_args(fields, session.get_interner())),
        &constructor_body,
        &Some(ast::TypeDef::from_expr(name_to_type(
            type_name,
            generics,
        ))),
        Uid::new_def(),
        session,
        used_names,
        imports,
        attributes,
        name_space,
    );

    if variant_info.is_none() {
        let struct_type = lower_ast_func(
            name,
            generics,
            None,  // args
            &ast::create_lang_item(
                LangItem::Dummy,
                name.span().into_fake(),
                session.get_interner(),
            ),
            &Some(ast::TypeDef::from_expr(ast::create_lang_item(
                LangItem::Type,
                name.span().into_fake(),
                session.get_interner(),
            ))),
            uid,
            session,
            used_names,
            imports,
            attributes,
            name_space,
        );

        let mut struct_type = struct_type?;
        struct_type.kind = FuncKind::StructDef;
        session.get_results_mut().insert(name.id(), struct_type);
    }

    let mut constructor = constructor?;
    constructor.kind = FuncKind::StructConstr;
    session.get_results_mut().insert(constructor_name.id(), constructor);

    Ok(())
}

fn fields_to_args(fields: &Vec<ast::FieldDef>, interner: &mut InternSession) -> Vec<ast::ArgDef> {
    let mut fields = fields.clone();
    sort_struct_fields(&mut fields, interner);

    fields.into_iter().map(
        |ast::FieldDef {
            name,
            ty,
            attributes,
        }| ast::ArgDef {
            name,
            ty: Some(ty),
            has_question_mark: false,
            attributes: attributes,
        }
    ).collect()
}

fn fields_to_values(fields: &Vec<ast::FieldDef>, interner: &mut InternSession) -> Vec<ast::Expr> {
    let mut fields = fields.clone();
    sort_struct_fields(&mut fields, interner);

    fields.into_iter().map(
        |ast::FieldDef {
            name, ..
        }| ast::Expr {
            kind: ast::ExprKind::Value(ast::ValueKind::Identifier(name.id())),
            span: name.span().into_fake(),
        }
    ).collect()
}

fn sort_struct_fields(
    fields: &mut Vec<ast::FieldDef>,
    interner: &mut InternSession,
) {
    fields.sort_by_key(|field| interner.unintern_string(field.name.id()).map(|id| id.to_vec()).unwrap_or(vec![]))
}

fn create_struct_body(
    values: Vec<ast::Expr>,
    span: SpanRange,
    interner: &mut InternSession,
) -> ast::Expr {
    ast::Expr {
        kind: ast::ExprKind::Call {
            func: Box::new(ast::create_lang_item(
                LangItem::StructBody,
                span,
                interner,
            )),
            args: values,
        },
        span,
    }
}

pub fn name_to_type(
    name: &IdentWithSpan,
    generics: &Vec<ast::GenericDef>,
) -> ast::Expr {
    if generics.is_empty() {
        ast::Expr {
            kind: ast::ExprKind::Value(ast::ValueKind::Identifier(name.id())),
            span: name.span().into_fake(),
        }
    }

    else {
        ast::Expr {
            kind: ast::ExprKind::Call {
                func: Box::new(name_to_type(name, &vec![])),
                args: generics.iter().map(
                    |generic| ast::Expr {
                        kind: ast::ExprKind::Value(ast::ValueKind::Identifier(generic.id())),
                        span: generic.span().into_fake(),
                    }
                ).collect(),
            },
            span: name.span().into_fake(),
        }
    }
}

fn wrap_struct_body_with_enum_body(
    struct_body: ast::Expr,
    variant_index: usize,
    interner: &mut InternSession,
) -> ast::Expr {
    let span = struct_body.span.into_fake();

    ast::Expr {
        kind: ast::ExprKind::Call {
            func: Box::new(ast::create_lang_item(
                LangItem::EnumBody,
                span,
                interner,
            )),
            args: vec![
                ast::Expr {
                    kind: ast::ExprKind::Value(ast::ValueKind::Number(interner.intern_numeric(SodigyNumber::from(variant_index as u32)))),
                    span,
                },
                struct_body,
            ],
        },
        span,
    }
}
