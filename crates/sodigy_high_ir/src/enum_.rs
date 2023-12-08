use crate::func::{FuncKind, lower_ast_func};
use crate::names::{IdentWithOrigin, NameSpace};
use crate::session::HirSession;
use sodigy_ast::{self as ast, IdentWithSpan};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;
use std::collections::{HashMap, HashSet};

/*
let enum Option<T> = { Some(T), None };
->
let Option<T>: Type = ...; let Some<T>(val: T): Option(T) = ...; let None<T>: Option<T> = ...;

let enum MsgKind<T> = { Quit, Event { kind: T, id: Int } };
->
let MsgKind<T>: Type = ...; let Quit<T>: MsgKind(T) = ...; let struct Event<T> = { kind: T, id: Int };
*/
pub fn lower_ast_enum(
    name: &IdentWithSpan,
    generics: &Vec<ast::GenericDef>,
    variants: &Vec<ast::VariantDef>,
    uid: Uid,
    session: &mut HirSession,
    used_names: &mut HashSet<IdentWithOrigin>,
    imports: &HashMap<InternedString, (SpanRange, Vec<IdentWithSpan>)>,
    decorators: &Vec<ast::Decorator>,
    doc: Option<InternedString>,
    name_space: &mut NameSpace,
) -> Result<(), ()> {
    let mut has_error = false;
    let parent_uid = uid;

    let mut variant_uids = Vec::with_capacity(variants.len());

    for ast::VariantDef {
        name, args, attributes,
    } in variants.iter() {
        let curr_uid = Uid::new_enum_variant();
        variant_uids.push(curr_uid);

        match args {
            // let None<T>: Option(T) = ...;
            ast::VariantKind::Empty => {
                if let Ok(mut f) = lower_ast_func(
                    name,
                    generics,
                    None,     // args
                    todo!(),  // return_val,
                    todo!(),  // return_ty,
                    uid,
                    session,
                    used_names,
                    imports,
                    todo!(),  // decorators
                    todo!(),  // doc
                    name_space,
                ) {
                    f.kind = FuncKind::EnumVariant { parent: parent_uid };
                    session.func_defs.insert(name.id(), f);
                } else {
                    has_error = true;
                }
            },
            // let Some<T>(val: T): Option(T) = ...;
            ast::VariantKind::Tuple(types) => {
                let args = types.iter().enumerate().map(
                    |(ind, ty)| ast::ArgDef {
                        name: session.make_nth_arg_name(ind),
                        ty: Some(ty.clone()),
                        has_question_mark: false,
                    }
                ).collect::<Vec<ast::ArgDef>>();

                if let Ok(f) = lower_ast_func(
                    name,
                    generics,
                    Some(&args),
                    todo!(),  // return_val,
                    todo!(),  // return_ty,
                    uid,
                    session,
                    used_names,
                    imports,
                    todo!(),  // decorators
                    todo!(),  // doc
                    name_space,
                ) {
                    f.kind = FuncKind::EnumVariant { parent: parent_uid };
                    session.func_defs.insert(name.id(), f);
                }

                else {
                    has_error = true;
                }
            },
            ast::VariantKind::Struct(_) => todo!(),
        }
    }

    // let Option<T>: Type = ...;
    if let Ok(mut f) = lower_ast_func(
        name,
        generics,
        None,     // args
        todo!(),  // return_val
        todo!(),  // return_ty
        uid,
        session,
        used_names,
        imports,
        decorators,
        doc,
        name_space,
    ) {
        f.kind = FuncKind::Enum { variants: variant_uids };
        session.func_defs.insert(name.id(), f);
    }

    else {
        has_error = true;
    }

    if has_error {
        Err(())
    }

    else {
        Ok(())
    }
}
