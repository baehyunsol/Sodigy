use crate::func::{FuncKind, lower_ast_func};
use crate::names::{IdentWithOrigin, NameSpace};
use crate::session::HirSession;
use crate::struct_::{lower_ast_struct, name_to_type};
use sodigy_ast::{self as ast, IdentWithSpan};
use sodigy_intern::{InternedString, InternSession};
use sodigy_lang_item::LangItem;
use sodigy_number::SodigyNumber;
use sodigy_session::SodigySession;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;
use std::collections::{HashMap, HashSet};

// when an enum has a struct variant,
// the variant is lowered to a struct, and goes through extra steps
// this info is needed for the extra steps
pub(crate) struct StructVariantInfo {
    pub variant_index: usize,
    pub parent_uid: Uid,
    pub parent_name: IdentWithSpan,
}

/*
let enum Option<T> = { Some(T), None };
->
let Option<T>: Type = @@dummy;
let Some<T>(val: T): Option(T) = @@enum_variant_body(
    0,    # variant index
    val,  # variant value
);

# Do we need <T> when this variant has no value?
let None<T>: Option(T) = @@enum_variant_body(
    1,   # variant index
    0,   # variant value: the compiler guarantees that this field is never read
);

for `Option<T>`, `Option` and `Option(Int)` is valid, but `Option()` is not. See the documents for the generics.

let enum MsgKind<T> = { Quit, Event { kind: T, id: Int } };
->
let MsgKind<T>: Type = @@dummy;
let Quit<T>: MsgKind(T) = @@enum_variant_body(
    0,
    0,
);

let struct @@MsgKind@@variant@@Event<T> = { kind: T, id: Int };
# which is lowered to
let @@struct_constructor_@@MsgKind@@variant@@Event<T>(data: T, id: Int): MsgKind(T) = @@enum_variant_body(
    1,
    @@struct_body(data, id),
);
*/

pub fn lower_ast_enum(
    enum_name: &IdentWithSpan,
    generics: &Vec<ast::GenericDef>,
    variants: &Vec<ast::VariantDef>,
    uid: Uid,
    session: &mut HirSession,
    used_names: &mut HashSet<IdentWithOrigin>,
    imports: &HashMap<InternedString, (SpanRange, Vec<IdentWithSpan>)>,
    attributes: &Vec<ast::Attribute>,
    name_space: &mut NameSpace,
) -> Result<(), ()> {
    let mut has_error = false;
    let parent_uid = uid;
    let mut variant_uids = Vec::with_capacity(variants.len());

    for (
        variant_index,
        ast::VariantDef {
            name: variant_name, args, attributes,
        },
    ) in variants.iter().enumerate() {
        let variant_uid = Uid::new_enum_variant();
        variant_uids.push(variant_uid);

        match args {
            // let None<T>: Option(T) = ...;
            ast::VariantKind::Empty => {
                let variant_func_name = add_enum_variant_prefix(
                    enum_name,
                    variant_name,
                    session.get_interner(),
                );

                if let Ok(mut f) = lower_ast_func(
                    &variant_func_name,
                    generics,
                    None,     // args
                    &create_enum_variant_body(
                        variant_index,
                        &[],
                        variant_name.span().into_fake(),
                        session.get_interner(),
                    ),
                    &Some(ast::TypeDef::from_expr(name_to_type(
                        enum_name,
                        generics,
                    ))),
                    variant_uid,
                    session,
                    used_names,
                    imports,
                    attributes,
                    name_space,
                ) {
                    f.kind = FuncKind::EnumVariant { parent: parent_uid };
                    session.get_results_mut().insert(variant_func_name.id(), f);
                } else {
                    has_error = true;
                }
            },
            // let Some<T>(val: T): Option(T) = ...;
            ast::VariantKind::Tuple(types) => {
                let args = types.iter().enumerate().map(
                    |(index, ty)| ast::ArgDef {
                        name: session.make_nth_arg_name(index, variant_name.span().into_fake()),
                        ty: Some(ty.clone()),
                        has_question_mark: false,
                        attributes: vec![],
                    }
                ).collect::<Vec<ast::ArgDef>>();
                let variant_func_name = add_enum_variant_prefix(
                    enum_name,
                    variant_name,
                    session.get_interner(),
                );

                if let Ok(mut f) = lower_ast_func(
                    &variant_func_name,
                    generics,
                    Some(&args),
                    &create_enum_variant_body(
                        variant_index,
                        &args,
                        variant_name.span().into_fake(),
                        session.get_interner(),
                    ),
                    &Some(ast::TypeDef::from_expr(name_to_type(
                        enum_name,
                        generics,
                    ))),
                    variant_uid,
                    session,
                    used_names,
                    imports,
                    attributes,
                    name_space,
                ) {
                    f.kind = FuncKind::EnumVariant { parent: parent_uid };
                    session.get_results_mut().insert(variant_func_name.id(), f);
                }

                else {
                    has_error = true;
                }
            },
            // let struct Event<T> = { kind: T, id: Int };
            ast::VariantKind::Struct(fields) => {
                if let Err(_) = lower_ast_struct(
                    &add_enum_struct_prefix(
                        enum_name,
                        variant_name,
                        session.get_interner(),
                    ),
                    generics,
                    fields,
                    variant_uid,
                    session,
                    used_names,
                    imports,
                    attributes,
                    name_space,
                    Some(StructVariantInfo {
                        parent_name: *enum_name,
                        parent_uid,
                        variant_index,
                    }),
                ) {
                    has_error = true;
                }
            },
        }
    }

    // let Option<T>: Type = ...;
    if let Ok(mut f) = lower_ast_func(
        enum_name,
        generics,
        None,     // args
        &ast::create_lang_item(
            LangItem::Dummy,
            enum_name.span().into_fake(),
            session.get_interner(),
        ),
        &Some(ast::TypeDef::from_expr(ast::create_lang_item(
            LangItem::Type,
            enum_name.span().into_fake(),
            session.get_interner(),
        ))),
        uid,
        session,
        used_names,
        imports,
        attributes,
        name_space,
    ) {
        f.kind = FuncKind::Enum { variants: variant_uids };
        session.get_results_mut().insert(enum_name.id(), f);
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

// there's another layer for the names of enum variants:
// 1. if two enums have variants with the same name, there could be a collision
// 2. if there's a user function whose name is the same as the name of the variant, that's also a collision
fn add_enum_variant_prefix(
    enum_name: &IdentWithSpan,
    variant_name: &IdentWithSpan,
    interner: &mut InternSession,
) -> IdentWithSpan {
    let variant_span = variant_name.span().into_fake();
    let enum_name = interner.unintern_string(enum_name.id()).unwrap().to_vec();
    let variant_name = interner.unintern_string(variant_name.id()).unwrap().to_vec();

    IdentWithSpan::new(
        interner.intern_string(
            vec![
                b"@@enum_".to_vec(),
                enum_name,
                b"@@variant_".to_vec(),
                variant_name,
            ].concat(),
        ),
        variant_span,
    )
}

// for a struct variant of an enum: it has to change its name in order to avoid name collisions
fn add_enum_struct_prefix(
    enum_name: &IdentWithSpan,
    variant_name: &IdentWithSpan,
    interner: &mut InternSession,
) -> IdentWithSpan {
    let variant_span = variant_name.span().into_fake();
    let enum_name = interner.unintern_string(enum_name.id()).unwrap().to_vec();
    let variant_name = interner.unintern_string(variant_name.id()).unwrap().to_vec();

    IdentWithSpan::new(
        interner.intern_string(
            vec![
                b"@@".to_vec(),
                enum_name,
                b"@@variant".to_vec(),
                b"@@".to_vec(),
                variant_name,
            ].concat(),
        ),
        variant_span,
    )
}

/*
# None
@@enum_variant_body(
    index,
    0,  # dummy
)

# Some(x)
@@enum_variant_body(
    index,
    x,
)

# Multi(x, y)
@@enum_variant_body(
    index,
    (x, y),  // into_tuple
)
*/
fn create_enum_variant_body(
    variant_index: usize,
    args: &[ast::ArgDef],
    span: SpanRange,
    interner: &mut InternSession,
) -> ast::Expr {
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
                match args.len() {
                    // TODO: I'm not sure how I would implement the later passes, but I guess `()` would be cheaper than `0`
                    0 => ast::Expr {  // dummy value. The compiler guarantees that it's not read by anyone
                        kind: ast::ExprKind::Value(ast::ValueKind::Number(interner.intern_numeric(SodigyNumber::from(0)))),
                        span,
                    },
                    1 => ast::Expr {
                        kind: ast::ExprKind::Value(ast::ValueKind::Identifier(args[0].name.id())),
                        span: *args[0].name.span(),
                    },
                    _ => ast::Expr {
                        kind: ast::ExprKind::Value(ast::ValueKind::Tuple(args.iter().map(
                            |ast::ArgDef {
                                name, ..
                            }| ast::Expr {
                                kind: ast::ExprKind::Value(ast::ValueKind::Identifier(name.id())),
                                span,
                            }
                        ).collect())),
                        span,
                    },
                },
            ],
        },
        span,
    }
}
