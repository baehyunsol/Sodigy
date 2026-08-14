use crate::Session;
use sodigy_mir::{Callable, Expr, MacroKind, Type, get_def_span_from_id, render_type, type_of};
use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::intern_string;
use sodigy_token::Constant;

pub fn lower_macro(kind: &MacroKind, macro_span: &Span, session: &mut Session) -> Result<Expr, ()> {
    match kind {
        MacroKind::IncludeString { path } |
        MacroKind::IncludeBytes { path } => todo!(),
        MacroKind::TypeName { r#type } => {
            let r#type = render_type(
                r#type,
                false,  // verbose
                session.global_context.lang_items.unwrap(),
                &session.intermediate_dir,
                session.global_context.span_string_map.unwrap(),
            ).unwrap();  // It's an ICE if any of the spans are unavailable

            Ok(Expr::Constant(Constant::String {
                binary: false,
                s: intern_string(r#type.as_bytes(), &session.intermediate_dir).unwrap(),
                span: macro_span.clone(),
            }))
        },
        MacroKind::TypeNameOfValue { value } => {
            // We don't have to lower this value (e.g. `lower_expr(value)`) because... we're not gonna evaluate this!!
            let r#type = type_of(value, session.global_context.clone()).unwrap();
            let r#type = render_type(
                &r#type,
                false,  // verbose
                session.global_context.lang_items.unwrap(),
                &session.intermediate_dir,
                session.global_context.span_string_map.unwrap(),
            ).unwrap();  // It's an ICE if any of the spans are unavailable

            Ok(Expr::Constant(Constant::String {
                binary: false,
                s: intern_string(r#type.as_bytes(), &session.intermediate_dir).unwrap(),
                span: macro_span.clone(),
            }))
        },
        MacroKind::NumberOfVariants { r#type } |
        MacroKind::NameOfVariants { r#type } => match r#type {
            Type::Data { constructor_def_span, args, .. } => {
                let def_span = get_def_span_from_id(*constructor_def_span, args);

                match session.global_context.enum_shapes.unwrap().get(&def_span) {
                    Some(enum_shape) => match kind {
                        MacroKind::NumberOfVariants { .. } => Ok(Expr::Constant(Constant::Number {
                            n: InternedNumber::from_u32(enum_shape.variants.len() as u32, true),
                            span: macro_span.clone(),
                        })),
                        MacroKind::NameOfVariants { .. } => {
                            let mut variants = Vec::with_capacity(enum_shape.variants.len());

                            for variant in enum_shape.variants.iter() {
                                variants.push(Expr::Constant(Constant::String {
                                    binary: false,
                                    s: variant.name,
                                    span: Span::None,
                                }));
                            }

                            Ok(Expr::Call {
                                func: Callable::ListInit { group_span: macro_span.clone() },
                                args: variants,
                                arg_group_span: Span::None,
                                types: None,
                                given_keyword_args: vec![],
                            })
                        },
                        _ => unreachable!(),
                    },
                    None => todo!(),
                }
            },
            _ => todo!(),
        },
        MacroKind::NumberOfFields { r#type } |
        MacroKind::NameOfFields { r#type } => match r#type {
            Type::Data { constructor_def_span, args, .. } => {
                let def_span = get_def_span_from_id(*constructor_def_span, args);

                match session.global_context.struct_shapes.unwrap().get(&def_span) {
                    Some(struct_shape) => match kind {
                        MacroKind::NumberOfFields { .. } => Ok(Expr::Constant(Constant::Number {
                            n: InternedNumber::from_u32(struct_shape.fields.len() as u32, true),
                            span: macro_span.clone(),
                        })),
                        MacroKind::NameOfFields { .. } => {
                            let mut fields = Vec::with_capacity(struct_shape.fields.len());

                            for field in struct_shape.fields.iter() {
                                fields.push(Expr::Constant(Constant::String {
                                    binary: false,
                                    s: field.name,
                                    span: Span::None,
                                }));
                            }

                            Ok(Expr::Call {
                                func: Callable::ListInit { group_span: macro_span.clone() },
                                args: fields,
                                arg_group_span: Span::None,
                                types: None,
                                given_keyword_args: vec![],
                            })
                        },
                        _ => unreachable!(),
                    },
                    None => todo!(),
                }
            },
            _ => todo!(),
        },
        MacroKind::File => todo!(),
        MacroKind::ModulePath => todo!(),
        MacroKind::Line => todo!(),
        MacroKind::Column => todo!(),
    }
}
