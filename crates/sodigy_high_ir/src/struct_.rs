use crate::func::{FuncKind, lower_ast_func};
use crate::names::{IdentWithOrigin, NameSpace};
use crate::session::HirSession;
use sodigy_ast::{self as ast, IdentWithSpan};
use sodigy_intern::InternedString;
use sodigy_lang_item::LangItem;
use sodigy_session::SodigySession;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;
use std::collections::{HashMap, HashSet};

/*
let struct Message<T> = { data: T, id: Int };
->
let __init_Message<T>(data: T, id: Int): Message(T) = ...;
let Message<T>: Type = ...;

`Message { data: "", id: 0 }` is lowered to `__init_Message`.
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
) -> Result<(), ()> {
    let constructor_name = IdentWithSpan::new(
        session.add_prefix(name.id(), "@@__init_"),
        *name.span(),
    );
    let constructor = lower_ast_func(
        &constructor_name,
        generics,
        Some(&fields_to_args(fields)),
        &ast::create_lang_item(
            LangItem::Todo,
            name.span().into_fake(),
            session.get_interner(),
        ),
        &Some(ast::TypeDef::from_expr(name_to_type(
            name,
            generics,
        ))),
        Uid::new_def(),
        session,
        used_names,
        imports,
        attributes,
        name_space,
    );
    let struct_type = lower_ast_func(
        name,
        generics,
        None,  // args
        &ast::create_lang_item(
            LangItem::Todo,
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

    let mut constructor = constructor?;
    constructor.kind = FuncKind::StructConstr;

    let mut struct_type = struct_type?;
    struct_type.kind = FuncKind::StructDef;

    session.get_results_mut().insert(constructor_name.id(), constructor);
    session.get_results_mut().insert(name.id(), struct_type);

    Ok(())
}

fn fields_to_args(fields: &Vec<ast::FieldDef>) -> Vec<ast::ArgDef> {
    fields.iter().map(
        |ast::FieldDef {
            name,
            ty,
            attributes,
        }| ast::ArgDef {
            name: *name,
            ty: Some(ty.clone()),
            has_question_mark: false,
            attributes: attributes.clone(),
        }
    ).collect()
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
