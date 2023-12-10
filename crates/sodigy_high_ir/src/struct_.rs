use crate::func::{FuncKind, lower_ast_func};
use crate::names::{IdentWithOrigin, NameSpace};
use crate::session::HirSession;
use sodigy_ast::{self as ast, IdentWithSpan};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;
use std::collections::{HashMap, HashSet};

/*
let struct Message<T> = { data: T, id: Int };
->
let __init_Message<T>(data: T, id: Int): Message(T) = ...;
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
    if let Ok(mut f) = lower_ast_func(
        &IdentWithSpan::new(
            session.add_prefix(name.id(), "@@__init_"),
            *name.span(),
        ),
        generics,
        Some(&fields_to_args(fields)),
        todo!(),  // return_val
        todo!(),  // return_ty
        uid,
        session,
        used_names,
        imports,
        attributes,
        name_space,
    ) {
        f.kind = FuncKind::StructConstr;
        session.func_defs.insert(name.id(), f);

        Ok(())
    } else {
        Err(())
    }
}

fn fields_to_args(fields: &Vec<ast::FieldDef>) -> Vec<ast::ArgDef> {
    fields.iter().map(
        |ast::FieldDef {
            name, ty, attributes,
        }| ast::ArgDef {
            name: *name,
            ty: Some(ty.clone()),
            has_question_mark: false,
            attributes: attributes.clone(),
        }
    ).collect()
}
