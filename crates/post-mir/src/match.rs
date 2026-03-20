//! [This](http://moscova.inria.fr/~maranget/papers/ml05e-maranget.pdf) is an excellent paper. You should read this.
//! My implementation is based on this paper.
//!
//! I also got a lot of inspirations from the [rust compiler](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_pattern_analysis/usefulness/index.html).
//!
//! TODO: [This one](http://moscova.inria.fr/~maranget/papers/warn/index.html) also looks good, I have to read it.
//!
//! ## Example 1
//!
//! ```ignore
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
//! ```ignore
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
//! ```ignore
//! // Name bindings are unused, but I want to demo how name bindings are processed.
//! match foo() {
//!     (Some($a @ 0..40), _) => 1,
//!     (Some($b), _) => 2,
//!     (_, Some($c)) => 3,
//!     (None, $d) => 4,
//!     (_, $e @ None) => 5,
//!     $f => 6,
//! }
//! ```
//!
//! becomes
//!
//! ```ignore
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
//! 1. If an arm does not appear in the decision tree, the arm is unreachable.
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

use crate::Session;
use sodigy_error::{Error, ErrorKind, Warning, WarningKind};
use sodigy_hir::{LetOrigin, Pattern, PatternKind};
use sodigy_mir::{
    ArmSplit,
    Block,
    Callable,
    Expr,
    Let,
    Match,
    MatchArm,
    type_of,
};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_number::InternedNumber;
use sodigy_parse::{Field, RestPattern};
use sodigy_span::{RenderableSpan, Span, SpanDeriveKind};
use sodigy_string::{InternedString, intern_string};
use sodigy_token::Constant;
use std::collections::HashSet;
use std::collections::hash_map::{Entry, HashMap};

mod dump;
mod matrix;
mod range;
mod tree;

use matrix::{MatrixConstructor, MatrixRow, get_list_sub_matrix, get_matrix};
pub(crate) use range::{LiteralType, Range, filter_out_invalid_ranges, merge_conditions, remove_overlaps};
use tree::{
    DecisionTree,
    DecisionTreeNode,
    ExprConstructor,
    build_tree,
};

// This is like `sodigy_parse::Field`, but for pattern analysis.
#[derive(Clone, Debug)]
pub enum PatternField {
    Constructor,
    Name {
        name: InternedString,
        name_span: Span,
        dot_span: Span,
        is_from_alias: bool,
    },
    Index(i64),
    Range(i64, i64),
    ListIndex(i64),
    ListRange(i64, i64),
    EnumDiscriminant,
    EnumPayload,
    ListLength,
    ListElements,
}

pub(crate) fn lower_match(match_expr: &mut Match, session: &mut Session) -> Result<Expr, ()> {
    let scrutinee_type = type_of(&match_expr.scrutinee, session.global_context.clone()).expect("Internal Compiler Error: Type-check is complete, but it failed to solve an expression!");

    // We use `index: usize` of each arm as an id of the arm.
    let mut arms: Vec<(usize, MatchArm)> = split_or_patterns(&match_expr.arms);

    // We'll use this arm to check exhaustiveness.
    let (extra_arm_id, extra_arm) = (arms.len(), MatchArm {
        pattern: Pattern {
            name: None,
            name_span: None,
            kind: PatternKind::Wildcard(Span::None),
        },
        guard: None,
        value: Expr::dummy(),
        group_id: None,
    });
    arms.push((extra_arm_id, extra_arm));

    let matrix = get_matrix(&scrutinee_type, session);
    let borrowed_arms: Vec<(usize, &MatchArm)> = arms.iter().map(
        |(id, arm)| (*id, arm)
    ).collect();

    let mut tree = match build_tree(
        &mut 1,
        &matrix,
        &borrowed_arms,
        session,
    ) {
        DecisionTreeNode::Tree(tree) => tree,
        DecisionTreeNode::Leaf { .. } => unreachable!(),
    };

    check_unreachable_and_exhaustiveness(
        &tree,
        &borrowed_arms,
        match_expr.keyword_span,
        extra_arm_id,
        session,
    )?;

    tree.optimize();

    if session.match_dumps.is_some() {
        let dump = session.dump_decision_tree(&tree, &borrowed_arms);
        session.match_dumps.as_mut().unwrap().insert(match_expr.keyword_span, dump);
    }

    // We have to evaluate the scrutinee multiple times.
    // If it's `match (x, y) { .. }`, we convert this to `{ let s = (x, y); match s { .. } }`.
    // If it's `match x { .. }`, we don't have to introduce another name binding.
    let another_name_binding = IdentWithOrigin {
        id: intern_string(b"scrutinee", "").unwrap(),
        span: Span::None,
        def_span: match_expr.keyword_span.derive(SpanDeriveKind::MatchScrutinee(0)),
        origin: NameOrigin::Local {
            kind: NameKind::Let { is_top_level: false },
        },
    };
    let (scrutinee, needs_another_name_binding) = match match_expr.scrutinee.as_ref() {
        Expr::Ident(id) => (Expr::Ident(*id), false),
        _ => (Expr::Ident(another_name_binding), true),
    };

    session.add_type_info(another_name_binding.def_span, scrutinee_type);
    let tree_expr = tree.into_expr(&scrutinee, &borrowed_arms, session);

    let tree_expr = if needs_another_name_binding {
        // We have to bind the name!!
        Expr::Block(Block {
            group_span: Span::None,
            lets: vec![Let {
                keyword_span: Span::None,
                name: another_name_binding.id,
                name_span: another_name_binding.def_span,
                type_annot_span: None,
                value: *match_expr.scrutinee.clone(),
                origin: LetOrigin::Match,
            }],
            asserts: vec![],
            value: Box::new(tree_expr),
        })
    } else {
        tree_expr
    };
    Ok(tree_expr)
}

#[derive(Clone, Debug)]
pub enum PatternConstructor {
    Tuple(usize),
    DefSpan(Span),
    Range(Range),
    Or(Vec<PatternConstructor>),
    Wildcard,

    // You can get this with `PatternField::ListElements`.
    ListSubMatrix {
        rest: Option<RestPattern>,
        elements: Vec<Pattern>,
    },
}

// pattern: `($a @ 0..20, ())`, field: `._0.constructor`
// -> constructor: Range { Int, 0..20 }, name_binding: Some((a, name_span of a))
//
// pattern: `($a @ 0..20, ())`, field: `.constructor`
// -> constructor: Tuple(2), name_binding: None
//
// pattern: `$x @ ($y, ())`, field: `._0.constructor`
// -> constructor: Wildcard, name_binding: Some((y, name_span of y))
fn read_field_of_pattern(
    arm_id: usize,
    pattern: &Pattern,
    field: &[PatternField],
    session: &Session,
) -> (PatternConstructor, Vec<NameBinding>) {
    assert!(!field.is_empty(), "Internal Compiler Error");
    let mut curr_pattern = pattern;
    let wildcard = Pattern {
        name: None,
        name_span: None,
        kind: PatternKind::Wildcard(Span::None),
    };

    if field.len() > 1 {
        for f in field[..(field.len() - 1)].iter() {
            match f {
                PatternField::Constructor |
                PatternField::ListElements => {},
                PatternField::Name { .. } => todo!(),
                PatternField::Index(i) => match &curr_pattern.kind {
                    // hir must have lowered this variant to `PatternKind::List`.
                    PatternKind::Constant(Constant::String { .. }) => unreachable!(),

                    PatternKind::Tuple { elements, rest, .. } => {
                        if let Some(_) = rest {
                            // `(a, .. , b)` is just a syntax sugar for `(a, _, _, b)`.
                            // we have to desugar this at some point
                            todo!()
                        }

                        else if *i >= 0 {
                            match elements.get(*i as usize) {
                                Some(p) => {
                                    curr_pattern = p;
                                },
                                None => todo!(),  // err
                            }
                        }

                        else {
                            todo!()
                        }
                    },
                    PatternKind::NameBinding { .. } | PatternKind::Wildcard(_) => {
                        curr_pattern = &wildcard;
                    },
                    p => panic!("TODO: {p:?}, {f:?}"),
                },
                PatternField::ListIndex(i) => match &curr_pattern.kind {
                    PatternKind::List { elements, rest, .. } => {
                        let rest_index = match rest {
                            Some(rest) => rest.index,
                            None => elements.len(),
                        };

                        if *i >= 0 && (*i as usize) < rest_index {
                            curr_pattern = &elements[*i as usize];
                        }

                        else {
                            todo!()
                        }
                    },
                    PatternKind::NameBinding { .. } | PatternKind::Wildcard(_) => {
                        curr_pattern = &wildcard;
                    },
                    p => panic!("TODO: {p:?}, {f:?}"),
                },
                PatternField::Range(_, _) => todo!(),
                PatternField::ListRange(_, _) => todo!(),
                PatternField::EnumDiscriminant |
                PatternField::EnumPayload |
                PatternField::ListLength => unreachable!(),
            }
        }
    }

    let mut name_bindings = match (curr_pattern.name, curr_pattern.name_span) {
        (Some(name), Some(name_span)) => vec![NameBinding { name, name_span, offset: NameBindingOffset::None, id: arm_id }],
        _ => vec![],
    };

    if let PatternKind::NameBinding { id, span } = &curr_pattern.kind {
        name_bindings.push(NameBinding { name: *id, name_span: *span, offset: NameBindingOffset::None, id: arm_id });
    }

    let constructor = match field.last().unwrap() {
        PatternField::Constructor => match &curr_pattern.kind {
            PatternKind::Constant(Constant::Number { n, .. }) => PatternConstructor::Range(Range {
                r#type: if n.is_integer { LiteralType::Int } else { LiteralType::Number },
                lhs: Some(n.clone()),
                lhs_inclusive: true,
                rhs: Some(n.clone()),
                rhs_inclusive: true,
            }),
            PatternKind::Constant(Constant::Char { ch, .. }) => PatternConstructor::Range(Range {
                r#type: LiteralType::Char,
                lhs: Some(InternedNumber::from_u32(*ch, true)),
                lhs_inclusive: true,
                rhs: Some(InternedNumber::from_u32(*ch, true)),
                rhs_inclusive: true,
            }),
            PatternKind::Constant(Constant::String { .. }) => PatternConstructor::DefSpan(session.get_lang_item_span("type.List")),
            PatternKind::Tuple { elements, rest, .. } => {
                if let Some(_) = rest {
                    // `(a, .. , b)` is just a syntax sugar for `(a, _, _, b)`.
                    // we have to desugar this at some point
                    todo!()
                }

                else {
                    PatternConstructor::Tuple(elements.len())
                }
            },
            PatternKind::List { .. } => PatternConstructor::DefSpan(session.get_lang_item_span("type.List")),
            PatternKind::Range { lhs, rhs, is_inclusive, .. } => {
                let mut literal_type = None;
                let lhs = lhs.as_ref().map(
                    |lhs| match &lhs.kind {
                        PatternKind::Constant(Constant::Number { n, .. }) => {
                            literal_type = Some(if n.is_integer { LiteralType::Int } else { LiteralType::Number });
                            n.clone()
                        },
                        PatternKind::Constant(Constant::Char { ch, .. }) => {
                            literal_type = Some(LiteralType::Char);
                            InternedNumber::from_u32(*ch, true)
                        },
                        PatternKind::Constant(Constant::Byte { b, .. }) => {
                            literal_type = Some(LiteralType::Byte);
                            InternedNumber::from_u32(*b as u32, true)
                        },
                        _ => unreachable!(),
                    }
                );
                let rhs = rhs.as_ref().map(
                    |rhs| match &rhs.kind {
                        PatternKind::Constant(Constant::Number { n, .. }) => {
                            literal_type = Some(if n.is_integer { LiteralType::Int } else { LiteralType::Number });
                            n.clone()
                        },
                        PatternKind::Constant(Constant::Char { ch, .. }) => {
                            literal_type = Some(LiteralType::Char);
                            InternedNumber::from_u32(*ch, true)
                        },
                        PatternKind::Constant(Constant::Byte { b, .. }) => {
                            literal_type = Some(LiteralType::Byte);
                            InternedNumber::from_u32(*b as u32, true)
                        },
                        _ => unreachable!(),
                    }
                );

                PatternConstructor::Range(Range {
                    r#type: literal_type.unwrap(),
                    lhs,
                    lhs_inclusive: true,
                    rhs,
                    rhs_inclusive: *is_inclusive,
                })
            },
            PatternKind::Or { lhs, rhs, .. } => {
                let (lhs, _) = read_field_of_pattern(arm_id, lhs, &[PatternField::Constructor], session);
                let (rhs, _) = read_field_of_pattern(arm_id, rhs, &[PatternField::Constructor], session);
                PatternConstructor::Or(vec![lhs, rhs])
            },
            PatternKind::Wildcard(_) | PatternKind::NameBinding { .. } => PatternConstructor::Wildcard,
            _ => panic!("TODO: {curr_pattern:?}"),
        },
        PatternField::Index(_) => unreachable!(),
        PatternField::ListIndex(_) => unreachable!(),
        PatternField::ListLength => {
            let (lhs, lhs_inclusive, rhs, rhs_inclusive) = match &curr_pattern.kind {
                // hir must have lowered this variant to `PatternKind::List`.
                PatternKind::Constant(Constant::String { .. }) => unreachable!(),

                PatternKind::List { elements, rest, .. } => {
                    if let Some(_) = rest {
                        // a rest pattern can be an arbitrary number of elements
                        (Some(elements.len()), true, None, false)
                    }

                    else {
                        (Some(elements.len()), true, Some(elements.len()), true)
                    }
                },
                PatternKind::Wildcard(_) | PatternKind::NameBinding { .. } => (Some(0), true, None, false),
                _ => panic!("TODO: {curr_pattern:?}"),
            };

            PatternConstructor::Range(Range {
                r#type: LiteralType::Scalar,
                lhs: lhs.map(|lhs| InternedNumber::from_u32(lhs as u32, true)),
                lhs_inclusive,
                rhs: rhs.map(|rhs| InternedNumber::from_u32(rhs as u32, true)),
                rhs_inclusive,
            })
        },
        PatternField::ListElements => match &curr_pattern.kind {
            // hir must have lowered this variant to `PatternKind::List`.
            PatternKind::Constant(Constant::String { .. }) => unreachable!(),
            PatternKind::List { elements, rest, .. } => {
                if let Some(RestPattern { name: Some(name), name_span: Some(name_span), index, .. }) = rest {
                    name_bindings.push(NameBinding {
                        id: arm_id,
                        name: *name,
                        name_span: *name_span,
                        offset: NameBindingOffset::Slice(*index as i64, -((elements.len() - *index) as i64)),
                    });
                }

                PatternConstructor::ListSubMatrix {
                    elements: elements.clone(),
                    rest: rest.clone(),
                }
            },
            PatternKind::NameBinding { .. } | PatternKind::Wildcard(_) => PatternConstructor::Wildcard,
            _ => todo!(),
        },
        f => panic!("TODO: {f:?}"),
    };

    (constructor, name_bindings)
}

#[derive(Clone, Debug)]
pub struct NameBinding {
    id: usize,
    name: InternedString,
    name_span: Span,
    offset: NameBindingOffset,
}

#[derive(Clone, Debug)]
pub enum NameBindingOffset {
    None,

    // let $x + 1 = foo();
    Number(InternedNumber),

    // let [$x, $xs @ ..] = foo();
    Slice(i64, i64),
}

// The compiler inserted an extra arm `_ => { .. }` at the end of a match expression.
// If the extra arm is reachable, the match expression is not exhaustive.
fn check_unreachable_and_exhaustiveness(
    tree: &DecisionTree,
    arms: &[(usize, &MatchArm)],
    keyword_span: Span,
    extra_arm_id: usize,
    session: &mut Session,
) -> Result<(), ()> {
    let mut hidden_by = HashMap::new();
    let mut reachable_arms = HashSet::new();
    check_arm_reachability(tree, &mut hidden_by, &mut reachable_arms);

    for (arm_id, arm) in arms.iter() {
        if *arm_id == extra_arm_id {
            continue;
        }

        if !reachable_arms.contains(arm_id) {
            // It's WarningKind::UnreachableOrPattern
            if let Some(_) = arm.group_id && false {
                todo!()
            }

            else {
                let mut warning_spans = vec![];
                warning_spans.extend(hidden_by.get(arm_id).unwrap().iter().map(
                    |arm_id| RenderableSpan {
                        span: arms[*arm_id].1.pattern.error_span_wide(),
                        auxiliary: true,
                        // TODO: better error message?
                        note: Some(String::from("This arm makes the arm unreachable.")),
                    }
                ).collect::<Vec<_>>());
                warning_spans.push(RenderableSpan {
                    span: arms[*arm_id].1.pattern.error_span_wide(),
                    auxiliary: false,
                    note: Some(String::from("This arm is unreachable.")),
                });

                session.warnings.push(Warning {
                    kind: WarningKind::UnreachableMatchArm,
                    spans: warning_spans,
                    note: None,
                });
            }
        }
    }

    // TODO: we can calculate the set of unreachable values
    if reachable_arms.contains(&extra_arm_id) {
        session.errors.push(Error {
            kind: ErrorKind::NonExhaustiveArms,
            spans: keyword_span.simple_error(),
            note: None,
        });
        Err(())
    }

    else {
        Ok(())
    }
}

fn check_arm_reachability(
    tree: &DecisionTree,
    hidden_by: &mut HashMap<usize, HashSet<usize>>,
    reachable_arms: &mut HashSet<usize>,
) {
    for branch in tree.branches.iter() {
        match &branch.node {
            DecisionTreeNode::Tree(tree) => check_arm_reachability(tree, hidden_by, reachable_arms),
            DecisionTreeNode::Leaf { matched, unmatched } => {
                reachable_arms.insert(*matched);

                for unmatched_id in unmatched.iter() {
                    match hidden_by.entry(*unmatched_id) {
                        Entry::Occupied(mut e) => {
                            e.get_mut().insert(*matched);
                        },
                        Entry::Vacant(e) => {
                            e.insert([*matched].into_iter().collect());
                        },
                    }
                }
            },
        }
    }
}

fn to_field_expr(expr: &Expr, fields: &[PatternField], session: &Session) -> Expr {
    if let Some(list_index_at) = fields.iter().position(|field| matches!(field, PatternField::ListIndex(_))) {
        let pre = to_field_expr(expr, &fields[..list_index_at], session);
        let PatternField::ListIndex(index) = fields[list_index_at] else { unreachable!() };
        // TODO: maybe we need `Constant::Scalar`
        let index = Expr::Constant(Constant::Char { ch: u32::try_from(index).unwrap(), span: Span::None });
        let expr = Expr::Call {
            func: Callable::Static {
                def_span: session.get_lang_item_span("built_in.index_list"),
                span: Span::None,
            },
            args: vec![pre, index],
            arg_group_span: Span::None,
            types: None,
            given_keyword_args: vec![],
        };

        if list_index_at + 1 < fields.len() {
            return to_field_expr(&expr, &fields[(list_index_at + 1)..], session);
        }

        else {
            return expr;
        }
    }

    let fields: Vec<Field> = fields.iter().filter_map(
        |field| match field {
            PatternField::Constructor | PatternField::ListElements => None,
            PatternField::Name { name, name_span, dot_span, is_from_alias } => Some(Field::Name {
                name: *name,
                name_span: *name_span,
                dot_span: *dot_span,
                is_from_alias: *is_from_alias,
            }),
            PatternField::Index(i) => Some(Field::Index(*i)),
            PatternField::Range(a, b) => Some(Field::Range(*a, *b)),
            PatternField::ListLength => Some(Field::ListLength),
            _ => panic!("TODO: {field:?}"),
        }
    ).collect();

    if fields.is_empty() {
        expr.clone()
    }

    // `x.y.z.__LIST_LENGTH__.w` -> `list_length(x.y.z).w`
    else if let Some(list_length_at) = fields.iter().position(|field| matches!(field, Field::ListLength)) {
        let arg = if list_length_at == 0 {
            expr.clone()
        } else {
            Expr::Field {
                lhs: Box::new(expr.clone()),
                fields: fields[..list_length_at].to_vec(),
            }
        };

        let list_length = Expr::Call {
            func: Callable::Static {
                def_span : session.get_lang_item_span("built_in.len_list"),
                span: Span::None,
            },
            args: vec![arg],
            arg_group_span: Span::None,
            types: None,
            given_keyword_args: vec![],
        };

        if list_length_at == fields.len() - 1 {
            list_length
        }

        else {
            Expr::Field {
                lhs: Box::new(list_length),
                fields: fields[(list_length_at + 1)..].to_vec(),
            }
        }
    }

    else {
        Expr::Field {
            lhs: Box::new(expr.clone()),
            fields,
        }
    }
}

fn split_or_patterns(arms: &[MatchArm]) -> Vec<(usize, MatchArm)> {
    let mut result = Vec::with_capacity(arms.len());
    let mut arm_id = 0;

    for arm in arms.iter() {
        match arm.split_or_patterns() {
            ArmSplit::NoSplit(arm) => {
                result.push((arm_id, arm.clone()));
                arm_id += 1;
            },
            ArmSplit::Split(arms) => {
                for arm in arms.into_iter() {
                    result.push((arm_id, arm));
                    arm_id += 1;
                }
            },
        }
    }

    result
}
