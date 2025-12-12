//! [This](http://moscova.inria.fr/~maranget/papers/ml05e-maranget.pdf) is an excellent paper. You should read this.
//! My implementation is based on this paper.
//!
//! I also got a lot of inspirations from the [rust compiler](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_pattern_analysis/usefulness/index.html).
//!
//! TODO: [This one](http://moscova.inria.fr/~maranget/papers/warn/index.html) also looks good, I have to read it.
//!
//! ## Example 1
//!
//! ```
//! match (a, b, c) {
//!     (0, 0, _) => 1,
//!     (0, _, _) => 2,
//!     (_, _, 0) => 3,
//!     (_, _, _) => 4,
//! }
//! ```
//!
//! becomes
//!
//! ```
//! let scrutinee = (a, b, c);
//!
//! match scrutinee.constructor {
//!     // It's kinda type-checker.
//!     Tuple3 => match scrutinee._0.constructor {
//!         0 => match scrutinee._1.constructor {  // 1, 2, 3, 4
//!             0 => match scrutinee._2.constructor {  // 1, 2, 3, 4
//!                 0 => 1,
//!                 ..0 | 1.. => 2,
//!             },
//!             // Optimization: we can replace this entire arm with `_ => 2`.
//!             ..0 | 1.. => match scrutinee._2.constructor {  // 2, 3, 4
//!                 0 => 2,
//!                 ..0 | 1.. => 2,
//!             },
//!         },
//!         // Optimization: we can replace `..0 | 1..` with `_`
//!         ..0 | 1.. => match scrutinee._1.constructor {  // 3, 4
//!             0 => match scrutinee._2.constructor {  // 3, 4
//!                 0 => 3,
//!                 ..0 | 1.. => 4,
//!             },
//!             ..0 | 1.. => match scrutinee._2.constructor {  // 4
//!                 0 => 4,
//!                 ..0 | 1.. => 4,
//!             },
//!         },
//!     },
//! }
//! ```
//!
//! ## Example 2
//!
//! ```
//! // Name bindings are unused, but I want to demo how name bindings are processed.
//! match foo() {
//!     (Some(a @ 0..40), _) => 1,
//!     (Some(b), _) => 2,
//!     (_, Some(c)) => 3,
//!     (None, d) => 4,
//!     (_, e @ None) => 5,
//!     f => 6,
//! }
//! ```
//!
//! becomes
//!
//! ```
//! let scrutinee = foo();
//!
//! match scrutinee.constructor {
//!     #[bind(arm: 6, name: f)]
//!     Tuple2 => match scrutinee._0.constructor {
//!         // 3 and 5 may or may not have payload. But they're wildcards anyways,
//!         // so they always match.
//!         Some => match scrutinee._0.payload {  // 1, 2, 3, 5, 6
//!             #[bind(arm: 1, name: a)]
//!             #[bind(arm: 2, name: b)]
//!             0..40 => match scrutinee._1.constructor {  // 1, 2, 3, 5, 6
//!                 Some => match scrutinee._1.payload {  // 1, 2, 3, 6
//!                     _ => 1,
//!                 },
//!                 #[bind(arm: 5, name: e)]
//!                 None => match scrutinee._1.payload {  // 1, 2, 5, 6
//!                     _ => 1,
//!                 },
//!             },
//!             #[bind(arm: 2, name: b)]
//!             ..0 | 41.. => match scrutinee._1.constructor {  // 2, 3, 5, 6
//!                 Some => match scrutinee._1.payload {  // 2, 3, 6
//!                     _ => 2,
//!                 },
//!                 #[bind(arm: 5, name: e)]
//!                 None => match scrutinee._1.payload {  // 2, 5, 6
//!                     _ => 2,
//!                 },
//!             },
//!         },
//!         None => match scrutinee._0.payload {  // 3, 4, 5, 6
//!             // Since this variant doesn't have a payload, every pattern matches.
//!             _ => match scrutinee._1.constructor {  // 3, 4, 5, 6
//!                 #[bind(arm: 4, name: d)]
//!                 Some => match scrutinee._1.payload {  // 3, 4, 6
//!                     #[bind(arm: 3, name: c)]
//!                     _ => 3,
//!                 },
//!                 #[bind(arm: 4, name: d)]
//!                 #[bind(arm: 5, name: e)]
//!                 None => match scrutinee._1.payload {  // 4, 5, 6
//!                     _ => 4,
//!                 },
//!             },
//!         },
//!     },
//! }
//! ```
//!
//! ## Exhaustiveness and unreachable arms.
//!
//! 1. If an arm does not appear in the state machine, the arm is unreachable.
//! 2. We add a fake arm with a wildcard pattern before lowering it.
//!    If the fake arm is reachable, the match is not exhaustive.
//!
//! ## Constructors and fields
//!
//! Every value has a constructor and 0 or more fields. Each constructor has a fixed number
//! of fields. A pattern matches a value if their constructors are the same, and the fields
//! are the same, recursively.
//!
//! - Tuple
//!   - constructor: length of the tuple (`(,)`, `(,,)`, ...)
//!   - fields: elements
//! - Struct
//!   - constructor: def_span
//!   - fields
//! - Enum
//!   - constructor: def_span (of variant)
//!   - fields: TODO
//! - List/String/Bytes: TODO
//! - Int/Number/Char/Byte
//!   - constructor: the value
//!     - For example, `2` is a constructor of the value `2`.
//!   - fields: none
//!
//! Some special patterns have multiple constructors: ranges, wildcards, var-length lists and or-patterns.

use crate::PatternAnalysisError;
use sodigy_error::{Error, Warning};
use sodigy_hir::{Generic, Pattern, PatternKind, StructField};
use sodigy_mir::{
    Callable,
    Expr,
    Match,
    MatchArm,
    MatchFsm,
    Session as MirSession,
    Type,
    type_of,
};
use sodigy_number::InternedNumber;
use sodigy_parse::Field;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

pub fn lower_matches(mir_session: &mut MirSession) -> Result<(), ()> {
    let mut has_error = false;

    for r#let in mir_session.lets.iter_mut() {
        has_error |= lower_matches_expr_recursive(
            &mut r#let.value,
            &mir_session.types,
            &mir_session.struct_shapes,
            &mir_session.lang_items,
            &mut mir_session.errors,
            &mut mir_session.warnings,
        ).is_err();
    }

    for func in mir_session.funcs.iter_mut() {
        has_error |= lower_matches_expr_recursive(
            &mut func.value,
            &mir_session.types,
            &mir_session.struct_shapes,
            &mir_session.lang_items,
            &mut mir_session.errors,
            &mut mir_session.warnings,
        ).is_err();
    }

    for assert in mir_session.asserts.iter_mut() {
        if let Some(note) = &mut assert.note {
            has_error |= lower_matches_expr_recursive(
                note,
                &mir_session.types,
                &mir_session.struct_shapes,
                &mir_session.lang_items,
                &mut mir_session.errors,
                &mut mir_session.warnings,
            ).is_err();
        }

        has_error |= lower_matches_expr_recursive(
            &mut assert.value,
            &mir_session.types,
            &mir_session.struct_shapes,
            &mir_session.lang_items,
            &mut mir_session.errors,
            &mut mir_session.warnings,
        ).is_err();
    }

    if has_error {
        Err(())
    }

    else {
        Ok(())
    }
}

fn lower_matches_expr_recursive(
    expr: &mut Expr,
    types: &HashMap<Span, Type>,
    struct_shapes: &HashMap<Span, (Vec<StructField>, Vec<Generic>)>,
    lang_items: &HashMap<String, Span>,
    errors: &mut Vec<Error>,
    warnings: &mut Vec<Warning>,
) -> Result<(), ()> {
    match expr {
        Expr::Ident(_) |
        Expr::Number { .. } |
        Expr::String { .. } |
        Expr::Char { .. } |
        Expr::Byte { .. } => Ok(()),
        Expr::If(r#if) => match (
            lower_matches_expr_recursive(r#if.cond.as_mut(), types, struct_shapes, lang_items, errors, warnings),
            lower_matches_expr_recursive(r#if.true_value.as_mut(), types, struct_shapes, lang_items, errors, warnings),
            lower_matches_expr_recursive(r#if.false_value.as_mut(), types, struct_shapes, lang_items, errors, warnings),
        ) {
            (Ok(()), Ok(()), Ok(())) => Ok(()),
            _ => Err(()),
        },
        Expr::Block(block) => {
            let mut has_error = false;

            for r#let in block.lets.iter_mut() {
                has_error |= lower_matches_expr_recursive(
                    &mut r#let.value,
                    types,
                    struct_shapes,
                    lang_items,
                    errors,
                    warnings,
                ).is_err();
            }

            for assert in block.asserts.iter_mut() {
                if let Some(note) = &mut assert.note {
                    has_error |= lower_matches_expr_recursive(
                        note,
                        types,
                        struct_shapes,
                        lang_items,
                        errors,
                        warnings,
                    ).is_err();
                }

                has_error |= lower_matches_expr_recursive(
                    &mut assert.value,
                    types,
                    struct_shapes,
                    lang_items,
                    errors,
                    warnings,
                ).is_err();
            }

            has_error |= lower_matches_expr_recursive(
                &mut block.value,
                types,
                struct_shapes,
                lang_items,
                errors,
                warnings,
            ).is_err();

            if has_error {
                Err(())
            }

            else {
                Ok(())
            }
        },
        Expr::Match(r#match) => {
            let mut has_error = false;

            has_error |= lower_matches_expr_recursive(
                &mut r#match.scrutinee,
                types,
                struct_shapes,
                lang_items,
                errors,
                warnings,
            ).is_err();

            for arm in r#match.arms.iter_mut() {
                if let Some(guard) = &mut arm.guard {
                    has_error |= lower_matches_expr_recursive(
                        guard,
                        types,
                        struct_shapes,
                        lang_items,
                        errors,
                        warnings,
                    ).is_err();
                }

                has_error |= lower_matches_expr_recursive(
                    &mut arm.value,
                    types,
                    struct_shapes,
                    lang_items,
                    errors,
                    warnings,
                ).is_err();
            }

            match lower_match(
                r#match,
                types,
                struct_shapes,
                lang_items,
                errors,
                warnings,
            ) {
                Ok(lowered) => {
                    *expr = Expr::MatchFsm(lowered);
                },
                Err(()) => {
                    has_error = true;
                },
            }

            if has_error {
                Err(())
            }

            else {
                Ok(())
            }
        },
        Expr::MatchFsm(_) => unreachable!(),
        Expr::Call { func, args, .. } => {
            let mut has_error = false;

            match func {
                Callable::Dynamic(f) => {
                    if let Err(()) = lower_matches_expr_recursive(
                        f,
                        types,
                        struct_shapes,
                        lang_items,
                        errors,
                        warnings,
                    ) {
                        has_error = true;
                    }
                },
                _ => {},
            }

            for arg in args.iter_mut() {
                if let Err(()) = lower_matches_expr_recursive(
                    arg,
                    types,
                    struct_shapes,
                    lang_items,
                    errors,
                    warnings,
                ) {
                    has_error = true;
                }
            }

            if has_error {
                Err(())
            }

            else {
                Ok(())
            }
        },
        _ => panic!("TODO: {expr:?}"),
    }
}

fn lower_match(
    match_ast: &mut Match,
    types: &HashMap<Span, Type>,
    struct_shapes: &HashMap<Span, (Vec<StructField>, Vec<Generic>)>,
    lang_items: &HashMap<String, Span>,
    errors: &mut Vec<Error>,
    warnings: &mut Vec<Warning>,
) -> Result<MatchFsm, ()> {
    let scrutinee_type = type_of(
        &match_ast.scrutinee,
        types,
        struct_shapes,
        lang_items,
    ).expect("Internal Compiler Error: Type-check is complete, but it failed to solve an expression!");

    let matrix = get_matrix(&scrutinee_type, lang_items);
    let arms: Vec<&MatchArm> = match_ast.arms.iter().collect();

    for arm in match_ast.arms.iter() {
        println!("{arm:?}");
    }

    todo!()
}

#[derive(Clone, Debug)]
enum LiteralType {
    Int,
    Number,
    Byte,
    Char,
}

#[derive(Clone, Debug)]
enum Constructor {
    Tuple(usize),
    DefSpan(Span),
    Range {
        r#type: LiteralType,
        lhs: Option<InternedNumber>,
        lhs_inclusive: bool,
        rhs: Option<InternedNumber>,
        rhs_inclusive: bool,
    },
    Or(Vec<Constructor>),
    Wildcard,
}

// Int: [([constructor], Range { Int, -inf..inf })]
// Number: [([constructor], Range { Number, -inf..inf })]  // We don't care about its denom and numer!
// (Foo, Foo, Int): [  // struct Foo { f1: Bool, f2: Int }
//     ([constructor], Tuple(2)),
//     ([index(0), constructor], DefSpan(Foo)),
//     ([index(0), name(f1), constructor], Or(DefSpan(True), DefSpan(False))),
//     ([index(0), name(f1), payload], EnumPayload(Bool)),  // this is empty, but we'll optimize that later
//     ([index(0), name(f2), constructor], Range { Int, -inf..inf }),
//     ([index(1), constructor], DefSpan(Foo)),
//     ([index(1), name(f1), constructor], Or(DefSpan(True), DefSpan(False))),
//     ([index(1), name(f1), payload], EnumPayload(Bool)),  // this is empty, but we'll optimize that later
//     ([index(1), name(f2), constructor], Range { Int, -inf..inf }),
//     ([index(2), constructor], Range { Int, -inf..inf }),
// ]
// (Int, Int, Option<Int>): [
//     ([constructor], Tuple(3)),
//     ([index(0), constructor], Range { Int, -inf..inf }),
//     ([index(1), constructor], Range { Int, -inf..inf }),
//     ([index(2), constructor], Or(DefSpan(Some), DefSpan(None))),
//     ([index(2), payload], EnumPayload(Option)),
// ]
fn get_matrix(
    r#type: &Type,
    lang_items: &HashMap<String, Span>,
) -> Vec<(Vec<Field>, Constructor)> {
    match r#type {
        Type::Static { def_span, .. } => {
            // TODO: It's toooo inefficient to call `lang_items.get()` everytime.
            if def_span == lang_items.get("type.Int").unwrap() {
                vec![(
                    vec![Field::Constructor],
                    Constructor::Range {
                        r#type: LiteralType::Int,
                        lhs: None,
                        lhs_inclusive: false,
                        rhs: None,
                        rhs_inclusive: false,
                    },
                )]
            }

            else {
                todo!()
            }
        },
        Type::Unit(_) => vec![(vec![Field::Constructor], Constructor::Tuple(0))],
        Type::Never(_) => todo!(),
        Type::Param { r#type, args, .. } => match &**r#type {
            Type::Static { def_span, .. } => todo!(),
            Type::Unit(_) => {
                let mut result = vec![(vec![Field::Constructor], Constructor::Tuple(args.len()))];

                for (i, arg) in args.iter().enumerate() {
                    let mut arg_matrix = get_matrix(arg, lang_items);

                    for row in arg_matrix.iter_mut() {
                        row.0.insert(0, Field::Index(i as i64));
                    }

                    result.extend(arg_matrix);
                }

                result
            },
            _ => unreachable!(),
        },
        Type::Func { params, r#return, .. } => todo!(),
        Type::GenericDef { .. } |
        Type::Var { .. } |
        Type::GenericInstance { .. } => panic!("Internal Compiler Error: Type-infer is complete, but I found a type variable!"),
    }
}

// pattern: `(a @ 0..20, None)`, field: `._0.constructor`
// -> (Range { Int, 0..20 }, Some((a, def_span of a)))
//
// pattern: `(a @ 0..20, None)`, field: `.constructor`
// -> (Tuple(2), None)
//
// TODO: how about dollar-identifiers?
// TODO: how about infix-ops?
fn get_constructor_of_pattern(
    pattern: &Pattern,
    field: &[Field],
) -> Result<(Constructor, Option<(InternedString, Span)>), PatternAnalysisError> {
    assert!(!field.is_empty(), "Internal Compiler Error");
    let name_binding = match (pattern.name, pattern.name_span) {
        (Some(name), Some(name_span)) => Some((name, name_span)),
        _ => None,
    };

    match &field[0] {
        Field::Constructor => match &pattern.kind {
            PatternKind::Ident { id, span } => Ok((
                Constructor::Wildcard,
                Some((*id, *span)),
            )),
            _ => todo!(),
        },
        _ => todo!(),
    }
}

/*
TODO

struct StateMachine {
    value: Vec<Field>,
    transitions: Vec<Transition>,
}

struct Transition {
    condition: Constructor,
    guard: Option<Expr>,
    state: StateMachineOrArm,

    // If the condition is met, the value is bound to the name.
    // It's bound AFTER `value` is evaluated and BEFORE the transition.
    bindings: Vec<(Arm, Span)>,  // (arm_index, def_span of name)
}
*/
