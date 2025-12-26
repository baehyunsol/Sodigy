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
//! At the leaf node of each decision tree, there may be multiple arms. For example, in the last
//! leaf of the above example, there are 3 arms: 4, 5 and 6.
//! We check the arms from top to bottom (4 -> 5 -> 6). If an arm has no guard, the arm is
//! matched, and the remaining arms are ignored. If an arm has a guard, the arm becomes another
//! leaf node (guard + match), and it continues checking the other arms.
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
    Session as MirSession,
    Type,
    type_of,
};
use sodigy_number::{InternedNumber, InternedNumberValue};
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
        Expr::Path { lhs, .. } => lower_matches_expr_recursive(
            lhs,
            types,
            struct_shapes,
            lang_items,
            errors,
            warnings,
        ),
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
                    *expr = lowered;
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
) -> Result<Expr, ()> {
    let scrutinee_type = type_of(
        &match_ast.scrutinee,
        types,
        struct_shapes,
        lang_items,
    ).expect("Internal Compiler Error: Type-check is complete, but it failed to solve an expression!");

    // We use index (`usize`) of each arm as an id of the arm.
    let arms: Vec<(usize, &MatchArm)> = match_ast.arms.iter().enumerate().collect();
    let matrix = get_matrix(&scrutinee_type, lang_items);

    let fsm = build_state_machine(
        &matrix,
        &arms,
        errors,
        warnings,
    )?;

    // TODO: turn this into a block
    // 1. if there're name bindings, prepend `let t = scrutinee.field;`.
    // 2. append `if _ { _ } else if ..`
    //   - body of the if expression is the transited state (or an arm)
    panic!("TODO: {fsm:?}")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    Range(Range),
    Or(Vec<Constructor>),
    Wildcard,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Range {
    r#type: LiteralType,
    lhs: Option<InternedNumber>,
    lhs_inclusive: bool,
    rhs: Option<InternedNumber>,
    rhs_inclusive: bool,
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
                    Constructor::Range(Range {
                        r#type: LiteralType::Int,
                        lhs: None,
                        lhs_inclusive: false,
                        rhs: None,
                        rhs_inclusive: false,
                    }),
                )]
            }

            else {
                todo!()
            }
        },
        Type::Unit(_) => vec![(vec![Field::Constructor], Constructor::Tuple(0))],
        Type::Never(_) => todo!(),
        Type::Param { constructor, args, .. } => match &**constructor {
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

#[derive(Clone, Debug)]
pub struct DestructuredPattern<'p> {
    pub pattern: &'p Pattern,
    pub constructor: Constructor,
    pub name_binding: Option<(InternedString, Span)>,
    pub name_binding_offset: Option<InternedNumberValue>,
}

impl DestructuredPattern<'_> {
    pub fn get_name_binding(&self, id: usize) -> Option<NameBinding> {
        if let Some((name, name_span)) = self.name_binding {
            Some(NameBinding {
                id,
                name,
                name_span,
                offset: self.name_binding_offset.clone(),
            })
        }

        else {
            None
        }
    }
}

// pattern: `(a @ 0..20, ())`, field: `._0.constructor`
// -> constructor: Range { Int, 0..20 }, name_binding: Some((a, def_span of a))
//
// pattern: `(a @ 0..20, ())`, field: `.constructor`
// -> constructor: Tuple(2), name_binding: None
//
// pattern: `($x, ())`, field: `._0.constructor`
fn read_field_of_pattern<'p>(
    pattern: &'p Pattern,
    field: &[Field],
) -> Result<DestructuredPattern<'p>, PatternAnalysisError> {
    assert!(!field.is_empty(), "Internal Compiler Error");
    let name_binding = match (pattern.name, pattern.name_span) {
        (Some(name), Some(name_span)) => Some((name, name_span)),
        _ => None,
    };

    match &field[0] {
        Field::Constructor => match &pattern.kind {
            PatternKind::Ident { id, span } => Ok(DestructuredPattern {
                pattern,
                constructor: Constructor::Wildcard,
                name_binding: Some((*id, *span)),
                name_binding_offset: None,
            }),
            PatternKind::Number { n, .. } => Ok(DestructuredPattern {
                pattern,
                constructor: Constructor::Range(Range {
                    r#type: if n.is_integer { LiteralType::Int } else { LiteralType::Number },
                    lhs: Some(n.clone()),
                    lhs_inclusive: true,
                    rhs: Some(n.clone()),
                    rhs_inclusive: true,
                }),
                name_binding,
                name_binding_offset: None,
            }),
            PatternKind::String { binary, s, .. } => todo!(),
            PatternKind::Char { ch, .. } => todo!(),
            PatternKind::Byte { b, .. } => todo!(),
            PatternKind::Tuple { elements, rest, .. } => {
                if let Some(_) = rest {
                    // `(a, .. , b)` is just a syntax sugar for `(a, _, _, b)`.
                    // we have to desugar this at some point
                    todo!()
                }

                else {
                    Ok(DestructuredPattern {
                        pattern,
                        constructor: Constructor::Tuple(elements.len()),
                        name_binding,
                        name_binding_offset: None,
                    })
                }
            },
            PatternKind::Wildcard(_) => Ok(DestructuredPattern {
                pattern,
                constructor: Constructor::Wildcard,
                name_binding,
                name_binding_offset: None,
            }),
            _ => todo!(),
        },
        Field::Index(i) => match &pattern.kind {
            PatternKind::Tuple { elements, rest, .. } => {
                if let Some(_) = rest {
                    // `(a, .. , b)` is just a syntax sugar for `(a, _, _, b)`.
                    // we have to desugar this at some point
                    todo!()
                }

                else {
                    // TODO: handle negative indexes
                    match elements.get(*i as usize) {
                        Some(p) => read_field_of_pattern(p, &field[1..]),
                        None => todo!(),  // err
                    }
                }
            },
            PatternKind::Ident { .. } | PatternKind::Wildcard(_) => Ok(DestructuredPattern {
                pattern,
                constructor: Constructor::Wildcard,
                name_binding,
                name_binding_offset: None,
            }),
            _ => panic!("TODO: {pattern:?}"),
        },
        f => panic!("TODO: {f:?}"),
    }
}

// In this state, it reads `scrutinee.field` and transits to the next state.
// There must be exactly 1 `transition` whose `.condition` meets `scrutinee.field`.
// If there are more than 1 transition, that's an ICE.
//
// `field` is None if it doesn't have to check scrutinee (e.g. when the transition is
// based on the match guards.
#[derive(Clone, Debug)]
pub struct StateMachine {
    field: Option<Vec<Field>>,
    transitions: Vec<Transition>,
}

#[derive(Clone, Debug)]
pub struct Transition {
    condition: Constructor,
    guard: Option<Expr>,
    state: StateMachineOrArm,

    // If the condition is met, `scrutinee.field` is bound to the name.
    // It's bound AFTER `scrutinee.field` is evaluated and BEFORE the transition.
    name_bindings: Vec<NameBinding>,
}

#[derive(Clone, Debug)]
pub enum StateMachineOrArm {
    StateMachine(StateMachine),
    Arm(usize),
}

#[derive(Clone, Debug)]
struct NameBinding {
    id: usize,
    name: InternedString,
    name_span: Span,
    offset: Option<InternedNumberValue>,
}

fn build_state_machine(
    matrix: &[(Vec<Field>, Constructor)],
    arms: &[(usize, &MatchArm)],
    errors: &mut Vec<Error>,
    warnings: &mut Vec<Warning>,
) -> Result<StateMachineOrArm, ()> {
    if matrix.is_empty() {
        let mut transitions = vec![];

        for (id, arm) in arms.iter() {
            if let Some(guard) = &arm.guard {
                transitions.push((Some(guard.clone()), *id));
            }

            else {
                transitions.push((None, *id));
                break;
            }
        }

        match transitions.len() {
            0 => todo!(),
            1 => match &transitions[0] {
                (Some(guard), _) => todo!(),
                (None, id) => {
                    return Ok(StateMachineOrArm::Arm(*id));
                },
            },
            _ => {
                return Ok(StateMachineOrArm::StateMachine(StateMachine {
                    field: None,
                    transitions: transitions.into_iter().map(
                        |(guard, id)| Transition {
                            condition: Constructor::Wildcard,
                            guard,
                            state: StateMachineOrArm::Arm(id),
                            name_bindings: vec![],
                        }
                    ).collect(),
                }));
            },
        }
    }

    let mut destructured_patterns = Vec::with_capacity(arms.len());

    for (id, arm) in arms.iter() {
        match read_field_of_pattern(
            &arm.pattern,
            &matrix[0].0,
        ) {
            Ok(pattern) => {
                destructured_patterns.push((*id, *arm, pattern));
            },
            Err(e) => todo!(),  // who handles this?
        }
    }

    match &matrix[0].1 {
        Constructor::Tuple(s_l) => {
            let mut okay_patterns = vec![];
            let mut name_bindings = vec![];

            for (id, arm, pattern) in destructured_patterns.iter() {
                match &pattern.constructor {
                    Constructor::Tuple(p_l) => {
                        if s_l == p_l {
                            okay_patterns.push((*id, *arm));

                            if let Some(name_binding) = pattern.get_name_binding(*id) {
                                name_bindings.push(name_binding);
                            }
                        }

                        else {
                            errors.push(Error::todo(19198, "type errors in patterns", pattern.pattern.error_span_wide()));
                        }
                    },
                    Constructor::Wildcard => {
                        okay_patterns.push((*id, *arm));

                        if let Some(name_binding) = pattern.get_name_binding(*id) {
                            name_bindings.push(name_binding);
                        }
                    },
                    _ => {
                        errors.push(Error::todo(19199, "type errors in patterns", pattern.pattern.error_span_wide()));
                    },
                }
            }

            Ok(StateMachineOrArm::StateMachine(StateMachine {
                field: Some(matrix[0].0.clone()),

                // no branches
                transitions: vec![Transition {
                    condition: Constructor::Wildcard,
                    guard: None,
                    state: build_state_machine(
                        &matrix[1..],
                        &okay_patterns,
                        errors,
                        warnings,
                    )?,
                    name_bindings,
                }],
            }))
        },
        Constructor::Range(Range { r#type, .. }) => {
            let mut transitions_with_overlap: Vec<(Range, Vec<(usize, &MatchArm)>, Vec<NameBinding>)> = vec![];

            // default: wildcard
            transitions_with_overlap.push((
                Range {
                    r#type: *r#type,
                    lhs: None,
                    lhs_inclusive: false,
                    rhs: None,
                    rhs_inclusive: false,
                },
                vec![],
                vec![],
            ));

            for (id, arm, pattern) in destructured_patterns.iter() {
                match &pattern.constructor {
                    Constructor::Range(r) => {
                        if r.r#type != *r#type {
                            errors.push(Error::todo(19200, "type errors in patterns", pattern.pattern.error_span_wide()));
                        }

                        else {
                            let mut is_new = true;

                            for (br, arms, name_bindings) in transitions_with_overlap.iter_mut() {
                                if br == r {
                                    arms.push((*id, *arm));

                                    if let Some(name_binding) = pattern.get_name_binding(*id) {
                                        name_bindings.push(name_binding);
                                    }

                                    is_new = false;
                                    break;
                                }
                            }

                            if is_new {
                                let mut name_bindings = vec![];

                                if let Some(name_binding) = pattern.get_name_binding(*id) {
                                    name_bindings.push(name_binding);
                                }

                                transitions_with_overlap.push((r.clone(), vec![(*id, *arm)], name_bindings));
                            }
                        }
                    },
                    Constructor::Wildcard => {
                        for (_, arms, name_bindings) in transitions_with_overlap.iter_mut() {
                            arms.push((*id, *arm));

                            if let Some(name_binding) = pattern.get_name_binding(*id) {
                                name_bindings.push(name_binding);
                            }
                        }
                    },
                    _ => {
                        errors.push(Error::todo(19201, "type errors in patterns", pattern.pattern.error_span_wide()));
                    },
                }
            }

            // TODO: remove overlaps in transitions

            let mut transitions = Vec::with_capacity(transitions_with_overlap.len());
            let mut has_error = false;

            for (range, arms, name_bindings) in transitions_with_overlap.into_iter() {
                match build_state_machine(
                    &matrix[1..],
                    &arms,
                    errors,
                    warnings,
                ) {
                    Ok(state) => {
                        transitions.push(Transition {
                            condition: Constructor::Range(range),
                            guard: None,
                            state,
                            name_bindings,
                        });
                    },
                    Err(_) => {
                        has_error = true;
                    },
                }
            }

            if has_error {
                Err(())
            }

            else {
                Ok(StateMachineOrArm::StateMachine(StateMachine {
                    field: Some(matrix[0].0.clone()),
                    transitions,
                }))
            }
        },
        c => panic!("TODO: {c:?}"),
    }
}
