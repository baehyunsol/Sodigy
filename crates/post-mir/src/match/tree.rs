use super::{
    LiteralType,
    MatrixConstructor,
    MatrixRow,
    NameBinding,
    NameBindingOffset,
    PatternConstructor,
    PatternField,
    Range,
    filter_out_invalid_ranges,
    get_list_sub_matrix,
    merge_conditions,
    read_field_of_pattern,
    remove_overlaps,
    to_field_expr,
};
use crate::Session;
use sodigy_hir::LetOrigin;
use sodigy_mir::{Block, Callable, Expr, If, Let, MatchArm, type_of};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_span::{Span, SpanDeriveKind};
use sodigy_string::intern_string;
use sodigy_token::Constant;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;

// In this tree, it reads `scrutinee.field` and branches to the next tree (or leaf).
// `.condition` of each branch must be non-overlapping.
//
// `field` is None if it doesn't have to check scrutinee (e.g. when the branch is
// based on the match guards.
#[derive(Clone, Debug)]
pub struct DecisionTree {
    // It's used for spans. Just make sure that trees in a match expression have unique id.
    pub id: u32,

    pub field: Option<Vec<PatternField>>,
    pub branches: Vec<DecisionTreeBranch>,
}

impl DecisionTree {
    /// ```ignore
    /// match (x, y) {
    ///     (0, 0) => 0,
    ///     (0, $a) => a,
    ///     (1, 1) => 2,
    ///     ($a, _) => a,
    /// }
    /// ```
    /// ->
    /// ```ignore
    /// {
    ///     let scrutinee = (x, y);
    ///     let curr = scrutinee._0;
    ///     let a = curr;
    ///
    ///     if curr == 0 {
    ///         let curr = scrutinee._1;
    ///         let a = curr;
    ///
    ///         if curr == 0 {
    ///             0
    ///         }
    ///
    ///         else {
    ///             a
    ///         }
    ///     }
    ///
    ///     else if curr == 1 {
    ///         2
    ///     }
    ///
    ///     else {
    ///         a
    ///     }
    /// }
    /// ```
    pub fn into_expr(
        &self,
        scrutinee: &Expr,
        arms: &[(usize, &MatchArm)],
        session: &mut Session,
    ) -> Expr {
        // TODO: We need some kinda cache for the scrutinee.
        //       For example, if there's `let curr = scrutinee._0._0` and `let curr = scrutinee._0._0._1`,
        //       we don't have to evaluate `scrutinee._0._0` twice.
        let curr_field_name = intern_string(b"curr", "").unwrap();
        let curr_field_span = scrutinee.error_span_wide().derive(SpanDeriveKind::MatchScrutinee(self.id));
        let curr_field = Expr::Ident(IdentWithOrigin {
            id: curr_field_name,
            span: curr_field_span.clone(),
            def_span: curr_field_span.clone(),
            origin: NameOrigin::Local {
                kind: NameKind::Let { is_top_level: false },
            },
        });

        let mut lets = match &self.field {
            Some(field) => {
                let curr_value = to_field_expr(scrutinee, field, session);
                let curr_value_type = type_of(&curr_value, session.global_context.clone()).unwrap();
                session.add_type_info(&curr_field_span, curr_value_type);

                vec![Let {
                    keyword_span: Span::None,
                    name: curr_field_name,
                    name_span: curr_field_span.clone(),
                    type_annot_span: None,
                    value: curr_value,
                    origin: LetOrigin::Match,
                }]
            },
            None => vec![],
        };

        // Name bindings can be duplicated while constructing a tree.
        // We have to deduplicate them here.
        let mut name_binding_spans = HashSet::new();

        for DecisionTreeBranch { name_bindings, .. } in self.branches.iter() {
            for name_binding in name_bindings.iter() {
                if name_binding_spans.contains(&name_binding.name_span) {
                    continue;
                }

                let mut new_value = Expr::Ident(IdentWithOrigin {
                    id: curr_field_name,
                    span: Span::None,
                    def_span: curr_field_span.clone(),
                    origin: NameOrigin::Local {
                        kind: NameKind::Let { is_top_level: false },
                    },
                });

                match &name_binding.offset {
                    NameBindingOffset::None => {},
                    NameBindingOffset::Number(n) => todo!(),
                    NameBindingOffset::Slice(start, 0) => {
                        new_value = Expr::Call {
                            func: Callable::Static {
                                def_span: session.get_lang_item_span("built_in.slice_right_list"),
                                span: Span::None,
                            },
                            args: vec![
                                new_value,
                                Expr::Constant(Constant::Scalar(*start as u32)),
                            ],
                            arg_group_span: Span::None,
                            types: None,
                            given_keyword_args: vec![],
                        };
                    },
                    NameBindingOffset::Slice(start, end) => todo!(),
                };

                let new_value_type = type_of(&new_value, session.global_context.clone()).unwrap();
                session.add_type_info(&name_binding.name_span, new_value_type);

                lets.push(Let {
                    keyword_span: Span::None,
                    name: name_binding.name,
                    name_span: name_binding.name_span.clone(),
                    type_annot_span: None,
                    value: new_value,
                    origin: LetOrigin::Match,
                });
                name_binding_spans.insert(name_binding.name_span.clone());
            }
        }

        let value = branches_to_expr(
            &self.branches,
            scrutinee,
            &curr_field,
            arms,
            session,
        );

        Expr::Block(Block {
            group_span: Span::None,
            lets,
            asserts: vec![],
            value: Box::new(value),
        })
    }

    // 1. If all the leaves in a node have the same arm id (`matched` field),
    //    there's no need for branch.
    // 2. It changes the order of `branches`: an arm with the most expensive condition
    //    will go to the end.
    //
    // TODO: Let's say there are 3 branches. The first branch's node matches arm-0 and the other
    // nodes match arm-1. In this case, we'd better merge the second and the third nodes.
    pub fn optimize(&mut self) {
        for branch in self.branches.iter_mut() {
            match &mut branch.node {
                DecisionTreeNode::Tree(tree) => {
                    tree.optimize();
                    let mut matched_arm_ids = vec![];
                    let mut unmatched_arm_ids = vec![];
                    let mut has_name_bindings = false;
                    tree.collect_matched_arm_ids(&mut matched_arm_ids, &mut unmatched_arm_ids, &mut has_name_bindings);
                    matched_arm_ids = matched_arm_ids.into_iter().collect::<HashSet<_>>().into_iter().collect();
                    unmatched_arm_ids = unmatched_arm_ids.into_iter().collect::<HashSet<_>>().into_iter().collect();

                    if let Some(arm_id) = matched_arm_ids.get(0) && matched_arm_ids.len() == 1 && !has_name_bindings {
                        branch.node = DecisionTreeNode::Leaf {
                            matched: *arm_id,
                            unmatched: unmatched_arm_ids,
                        };
                    }
                },
                DecisionTreeNode::Leaf { .. } => {},
            }
        }

        // very naive heuristic: the more constructors you have in `Constructor::Or`,
        // the more expensive it is to evaluate the condition.
        self.branches.sort_by_key(
            |branch| match &branch.condition {
                ExprConstructor::Or(cs) => cs.len(),
                _ => 0,
            }
        );
    }

    fn collect_matched_arm_ids(
        &self,
        matched_arm_ids: &mut Vec<usize>,
        unmatched_arm_ids: &mut Vec<usize>,
        has_name_bindings: &mut bool,
    ) {
        for branch in self.branches.iter() {
            match &branch.node {
                DecisionTreeNode::Tree(tree) => {
                    tree.collect_matched_arm_ids(matched_arm_ids, unmatched_arm_ids, has_name_bindings);
                },
                DecisionTreeNode::Leaf { matched, unmatched } => {
                    matched_arm_ids.push(*matched);
                    unmatched_arm_ids.extend(unmatched);
                },
            }

            if !branch.name_bindings.is_empty() {
                *has_name_bindings = true;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DecisionTreeBranch {
    pub condition: ExprConstructor,

    pub guard: Option<Expr>,
    pub node: DecisionTreeNode,

    // If the condition is met, `scrutinee.field` is bound to the name.
    // It's bound AFTER `scrutinee.field` is evaluated and BEFORE the branch.
    pub name_bindings: Vec<NameBinding>,
}

fn branches_to_expr(
    branches: &[DecisionTreeBranch],
    scrutinee: &Expr,
    curr_field: &Expr,
    arms: &[(usize, &MatchArm)],
    session: &mut Session,
) -> Expr {
    match branches {
        [branch] => match &branch.node {
            DecisionTreeNode::Tree(tree) => tree.into_expr(scrutinee, arms, session),
            DecisionTreeNode::Leaf { matched, .. } => arms[*matched].1.value.clone(),
        },
        branches => Expr::If(If {
            if_span: Span::None,
            cond: Box::new(branch_condition_to_expr(&branches[0], curr_field, session)),
            else_span: Span::None,
            true_value: Box::new(branches_to_expr(&branches[0..1], scrutinee, curr_field, arms, session)),
            true_group_span: Span::None,
            false_value: Box::new(branches_to_expr(&branches[1..], scrutinee, curr_field, arms, session)),
            false_group_span: Span::None,
            from_short_circuit: None,
        }),
    }
}

fn branch_condition_to_expr(
    branch: &DecisionTreeBranch,
    curr_field: &Expr,
    session: &Session,
) -> Expr {
    match &branch.condition {
        ExprConstructor::Wildcard => match &branch.guard {
            Some(guard) => guard.clone(),
            None => true_value(session),
        },
        c => constructor_to_expr(c, curr_field, session),
    }
}

fn constructor_to_expr(
    constructor: &ExprConstructor,
    curr_field: &Expr,
    session: &Session,
) -> Expr {
    match constructor {
        ExprConstructor::Range(range) => {
            let (lang_item, operand) = match (range.lhs, range.rhs) {
                (Some(lhs), Some(rhs)) => match lhs.cmp(rhs, &session.intermediate_dir) {
                    Ordering::Equal if range.lhs_inclusive && range.rhs_inclusive => match range.r#type {
                        LiteralType::Int => (
                            "built_in.eq_int",
                            Expr::Constant(Constant::Number { n: lhs, span: Span::None }),
                        ),
                        LiteralType::Byte => (
                            "built_in.eq_scalar",
                            Expr::Constant(Constant::Byte { b: lhs.try_into().unwrap(), span: Span::None }),
                        ),
                        LiteralType::Char => (
                            "built_in.eq_scalar",
                            Expr::Constant(Constant::Char { ch: lhs.try_into().unwrap(), span: Span::None }),
                        ),
                        LiteralType::Scalar => (
                            "built_in.eq_scalar",
                            Expr::Constant(Constant::Scalar(lhs.try_into().unwrap())),
                        ),
                        _ => todo!(),
                    },
                    Ordering::Less => {
                        if range.r#type.is_int_like() && lhs.add_one() == rhs {
                            // `3 < x && x <= 4` is just `x == 4`
                            match (range.r#type, range.lhs_inclusive, range.rhs_inclusive) {
                                (LiteralType::Int, false, true) => ("built_in.eq_int", Expr::Constant(Constant::Number { n: rhs, span: Span::None })),
                                (LiteralType::Int, true, false) => ("built_in.eq_int", Expr::Constant(Constant::Number { n: lhs, span: Span::None })),
                                (LiteralType::Int, _, _) => {
                                    return true_value(session);
                                },
                                // we need `built_in.eq_scalar`
                                _ => todo!(),
                            }
                        }

                        // `0..10` is lowered to `0 <= x && x < 10`, which is then lowered to
                        // `if 0 <= x { x < 10 } else { False }`
                        else {
                            let (lhs, rhs) = match range.r#type {
                                LiteralType::Int | LiteralType::Number => (
                                    Expr::Constant(Constant::Number { n: lhs, span: Span::None }),
                                    Expr::Constant(Constant::Number { n: rhs, span: Span::None }),
                                ),
                                LiteralType::Byte => (
                                    Expr::Constant(Constant::Byte { b: lhs.try_into().unwrap(), span: Span::None }),
                                    Expr::Constant(Constant::Byte { b: rhs.try_into().unwrap(), span: Span::None }),
                                ),
                                LiteralType::Char => (
                                    Expr::Constant(Constant::Char { ch: lhs.try_into().unwrap(), span: Span::None }),
                                    Expr::Constant(Constant::Char { ch: rhs.try_into().unwrap(), span: Span::None }),
                                ),
                                LiteralType::Scalar => (
                                    Expr::Constant(Constant::Scalar(lhs.try_into().unwrap())),
                                    Expr::Constant(Constant::Scalar(rhs.try_into().unwrap())),
                                ),
                            };
                            let f1 = match (range.r#type, range.lhs_inclusive) {
                                (LiteralType::Int, true) => "fn.leq_int",
                                (LiteralType::Int, false) => "built_in.lt_int",
                                (LiteralType::Number, true) => "fn.leq_number",
                                (LiteralType::Number, false) => "fn.lt_number",
                                (LiteralType::Char | LiteralType::Byte | LiteralType::Scalar, true) => "fn.leq_scalar",
                                (LiteralType::Char | LiteralType::Byte | LiteralType::Scalar, false) => "built_in.lt_scalar",
                            };
                            let f2 = match (range.r#type, range.rhs_inclusive) {
                                (LiteralType::Int, true) => "fn.leq_int",
                                (LiteralType::Int, false) => "built_in.lt_int",
                                (LiteralType::Number, true) => "fn.leq_number",
                                (LiteralType::Number, false) => "fn.lt_number",
                                (LiteralType::Char | LiteralType::Byte | LiteralType::Scalar, true) => "fn.leq_scalar",
                                (LiteralType::Char | LiteralType::Byte | LiteralType::Scalar, false) => "built_in.lt_scalar",
                            };

                            return Expr::If(If {
                                if_span: Span::None,
                                cond: Box::new(Expr::Call {
                                    func: Callable::Static {
                                        def_span: session.get_lang_item_span(f1),
                                        span: Span::None,
                                    },
                                    args: vec![
                                        lhs,
                                        curr_field.clone(),
                                    ],
                                    arg_group_span: Span::None,
                                    types: None,
                                    given_keyword_args: vec![],
                                }),
                                else_span: Span::None,
                                true_value: Box::new(Expr::Call {
                                    func: Callable::Static {
                                        def_span: session.get_lang_item_span(f2),
                                        span: Span::None,
                                    },
                                    args: vec![
                                        curr_field.clone(),
                                        rhs,
                                    ],
                                    arg_group_span: Span::None,
                                    types: None,
                                    given_keyword_args: vec![],
                                }),
                                true_group_span: Span::None,
                                false_value: Box::new(Expr::Ident(IdentWithOrigin {
                                    id: intern_string(b"False", &session.intermediate_dir).unwrap(),
                                    span: Span::None,
                                    origin: NameOrigin::Foreign {
                                        kind: NameKind::EnumVariant {
                                            parent: session.get_lang_item_span("type.Bool"),
                                        },
                                    },
                                    def_span: session.get_lang_item_span("variant.Bool.False"),
                                })),
                                false_group_span: Span::None,
                                from_short_circuit: None,
                            });
                        }
                    },
                    // `4 < x && x < 3` is just impossible
                    // `3 < x && x <= 3` is just impossible
                    // TODO: It's likely to be a compiler bug. Maybe we should write some log here?
                    Ordering::Greater | Ordering::Equal => {
                        return false_value(session);
                    },
                },
                (Some(lhs), None) => match (range.r#type, range.lhs_inclusive) {
                    (LiteralType::Int, true) => ("built_in.geq_int", Expr::Constant(Constant::Number { n: lhs, span: Span::None })),
                    (LiteralType::Int, false) => ("built_in.gt_int", Expr::Constant(Constant::Number { n: lhs, span: Span::None })),
                    _ => todo!(),
                },
                (None, Some(rhs)) => match (range.r#type, range.rhs_inclusive) {
                    (LiteralType::Int, true) => ("built_in.leq_int", Expr::Constant(Constant::Number { n: rhs, span: Span::None })),
                    (LiteralType::Int, false) => ("built_in.lt_int", Expr::Constant(Constant::Number { n: rhs, span: Span::None })),
                    _ => todo!(),
                },
                (None, None) => {
                    return true_value(session);
                },
            };

            Expr::Call {
                func: Callable::Static {
                    def_span: session.get_lang_item_span(lang_item),
                    span: Span::None,
                },
                args: vec![
                    curr_field.clone(),
                    operand,
                ],
                arg_group_span: Span::None,
                types: None,
                given_keyword_args: vec![],
            }
        },
        ExprConstructor::Or(cs) => constructors_to_expr(cs, curr_field, session),
        ExprConstructor::Wildcard => true_value(session),
    }
}

fn constructors_to_expr(
    constructors: &[ExprConstructor],
    curr_field: &Expr,
    session: &Session,
) -> Expr {
    match constructors.len() {
        1 => constructor_to_expr(&constructors[0], curr_field, session),
        _ => Expr::If(If {
            if_span: Span::None,
            cond: Box::new(constructor_to_expr(&constructors[0], curr_field, session)),
            else_span: Span::None,
            true_value: Box::new(Expr::Ident(IdentWithOrigin {
                id: intern_string(b"True", &session.intermediate_dir).unwrap(),
                span: Span::None,
                origin: NameOrigin::Foreign {
                    kind: NameKind::EnumVariant {
                        parent: session.get_lang_item_span("type.Bool"),
                    },
                },
                def_span: session.get_lang_item_span("variant.Bool.True"),
            })),
            true_group_span: Span::None,
            false_value: Box::new(constructors_to_expr(&constructors[1..], curr_field, session)),
            false_group_span: Span::None,
            from_short_circuit: None,
        }),
    }
}

#[derive(Clone, Debug)]
pub enum DecisionTreeNode {
    Tree(DecisionTree),

    // If there are multiple arms that can reach here, the first
    // arm is matched, and the remainings are pushed to `unmatched`.
    // `unmatched` is later used for warning messages, if necessary.
    Leaf {
        matched: usize,
        unmatched: Vec<usize>,
    },
}

pub(crate) fn build_tree(
    tree_id: &mut u32,
    matrix: &[MatrixRow],
    arms: &[(usize, &MatchArm)],
    session: &mut Session,
) -> DecisionTreeNode {
    if matrix.is_empty() {
        let mut branches = vec![];
        let mut unmatched_ids = vec![];

        for (i, (id, arm)) in arms.iter().enumerate() {
            if let Some(guard) = &arm.guard {
                branches.push((Some(guard.clone()), *id));
            }

            else {
                branches.push((None, *id));

                if i + 1 < arms.len() {
                    unmatched_ids = arms[(i + 1)..].iter().map(|(id, _)| *id).collect();
                }

                break;
            }
        }

        match branches.len() {
            0 => todo!(),
            1 => match &branches[0] {
                (Some(guard), _) => todo!(),
                (None, id) => {
                    return DecisionTreeNode::Leaf {
                        matched: *id,
                        unmatched: unmatched_ids,
                    };
                },
            },
            _ => {
                *tree_id += 1;
                return DecisionTreeNode::Tree(DecisionTree {
                    id: *tree_id,
                    field: None,
                    branches: branches.into_iter().map(
                        |(guard, id)| DecisionTreeBranch {
                            condition: ExprConstructor::Wildcard,
                            guard,
                            node: DecisionTreeNode::Leaf {
                                matched: id,
                                unmatched: unmatched_ids.clone(),
                            },
                            name_bindings: vec![],
                        }
                    ).collect(),
                });
            },
        }
    }

    let mut destructured_patterns = Vec::with_capacity(arms.len());

    for (id, arm) in arms.iter() {
        let (pattern, name_bindings) = read_field_of_pattern(
            *id,
            &arm.pattern,
            &matrix[0].field,
            session,
        );
        destructured_patterns.push((*id, *arm, pattern, name_bindings));
    }

    match &matrix[0].constructor {
        MatrixConstructor::Tuple(s_l) => {
            let mut okay_patterns = vec![];
            let mut name_bindings = vec![];

            for (id, arm, pattern, name_bindings_) in destructured_patterns.into_iter() {
                name_bindings.extend(name_bindings_);

                match pattern {
                    PatternConstructor::Tuple(p_l) if *s_l == p_l => {
                        okay_patterns.push((id, arm));
                    },
                    PatternConstructor::Wildcard => {
                        okay_patterns.push((id, arm));
                    },
                    p => panic!("TODO: {p:?}"),
                }
            }

            *tree_id += 1;
            DecisionTreeNode::Tree(DecisionTree {
                id: *tree_id,
                field: Some(matrix[0].field.clone()),

                // no branches
                branches: vec![DecisionTreeBranch {
                    condition: ExprConstructor::Wildcard,
                    guard: None,
                    node: build_tree(
                        tree_id,
                        &matrix[1..],
                        &okay_patterns,
                        session,
                    ),
                    name_bindings,
                }],
            })
        },
        MatrixConstructor::DefSpan(def_span) => {
            let mut okay_patterns = vec![];
            let mut name_bindings = vec![];

            for (id, arm, pattern, name_bindings_) in destructured_patterns.into_iter() {
                name_bindings.extend(name_bindings_);

                match &pattern {
                    PatternConstructor::DefSpan(d) if d == def_span => {
                        okay_patterns.push((id, arm));
                    },
                    PatternConstructor::Wildcard => {
                        okay_patterns.push((id, arm));
                    },
                    _ => todo!(),
                }
            }

            *tree_id += 1;
            DecisionTreeNode::Tree(DecisionTree {
                id: *tree_id,
                field: Some(matrix[0].field.clone()),

                // no branches
                branches: vec![DecisionTreeBranch {
                    condition: ExprConstructor::Wildcard,
                    guard: None,
                    node: build_tree(
                        tree_id,
                        &matrix[1..],
                        &okay_patterns,
                        session,
                    ),
                    name_bindings,
                }],
            })
        },
        MatrixConstructor::Range(r @ Range { r#type, .. }) => {
            let mut branches_with_overlap: Vec<(Range, (Vec<(usize, &MatchArm)>, Vec<NameBinding>))> = vec![];
            destructured_patterns = split_or_patterns(destructured_patterns);

            // default: wildcard
            branches_with_overlap.push((r.clone(), (vec![], vec![])));

            for (id, arm, pattern, name_bindings_) in destructured_patterns.into_iter() {
                match &pattern {
                    PatternConstructor::Range(r) => {
                        if r.r#type != *r#type {
                            panic!("ICE!")
                        }

                        else {
                            let mut is_new = true;

                            for (br, (arms, name_bindings)) in branches_with_overlap.iter_mut() {
                                if br == r {
                                    arms.push((id, arm));
                                    name_bindings.extend(name_bindings_.clone());
                                    is_new = false;
                                    break;
                                }
                            }

                            if is_new {
                                branches_with_overlap.push((r.clone(), (vec![(id, arm)], name_bindings_)));
                            }
                        }
                    },
                    PatternConstructor::Wildcard => {
                        for (_, (arms, name_bindings)) in branches_with_overlap.iter_mut() {
                            arms.push((id, arm));
                            name_bindings.extend(name_bindings_.clone());
                        }
                    },
                    _ => panic!("TODO: {pattern:?}"),
                }
            }

            // Let's say the match expression is
            // ```
            // match _ {
            //     0 => 0,
            //     2 => 1,
            //     1..5 => 2,
            //     3..10 => 3,
            //     _ => 4,
            // }
            // ```
            //
            // Then `branches_with_overlap` looks like
            //
            // ```
            // [
            //     (range: (-inf, +inf), arms: [4]),
            //     (range: [0, 0],       arms: [0, 4]),
            //     (range: [2, 2],       arms: [1, 4]),
            //     (range: [1, 5),       arms: [2, 4]),
            //     (range: [3, 10),      arms: [3, 4]),
            // ]
            // ```
            //
            // It just naively translated ranges and allocated arm ids.
            // There're small optimizations, tho:
            //    1. wildcard ranges are allocated to every branches
            //    2. if two arms have exactly the same ranges, they're in the same branch (not shown in the example above)

            let branches_without_overlap = remove_overlaps(branches_with_overlap, &session.intermediate_dir);

            // after `remove_overlaps()`, `branches_without_overlap` looks like below:
            //
            // ```
            // [
            //     (range: (-inf, 0),  arms: [4]),
            //     (range: [0, 0],     arms: [0, 4]),
            //     (range: [1, 2),     arms: [2, 4]),
            //     (range: [2, 2],     arms: [1, 2, 4]),
            //     (range: [3, 5),     arms: [2, 3, 4]),
            //     (range: [5, 10),    arms: [3, 4]),
            //     (range: [10, +inf), arms: [4]),
            // ]
            // ```
            //
            // Now the ranges are splitted: no ranges are overlapping.
            // For example, if scrutinee is 2, it can match arm-1 (whose range is [2, 2]),
            // arm-2 (whose range is [1, 5)), or arm-4 (whose range is (-inf, +inf)).
            // So the range [2,2] in the above value has arms [1, 2, 4].

            let branches_without_overlap = filter_out_invalid_ranges(branches_without_overlap, &session.intermediate_dir);

            // Unlike int, some `LiteralType`s (like `Char` or `Byte`) have invalid ranges.
            // For example, a byte has to be in range 0..256 and a char has to be in
            // range 0..0xd800 | 0xe000..0x110000. `filter_out_invalid_ranges` will filter out
            // such ranges. In this example, `filter_out_invalid_ranges` does nothing because
            // `LiteralType::Int` has no invalid ranges.

            let branches_without_overlap = merge_conditions(branches_without_overlap);

            // after `merge_conditions()`, `branches_without_overlap` looks like below:
            //
            // ```
            // [
            //     (condition: (-inf, 0) | [10, +inf), arms: [4]),
            //     (condition: [0, 0],  arms: [0, 4]),
            //     (condition: [1, 2),  arms: [2, 4]),
            //     (condition: [2, 2],  arms: [1, 2, 4]),
            //     (condition: [3, 5),  arms: [2, 3, 4]),
            //     (condition: [5, 10), arms: [3, 4]),
            // ]
            // ```
            //
            // In the previous step 2 ranges (-inf, 0) and [10, +inf) were both pointing to arms [4],
            // so they're grouped.

            // println!(
            //     "{:?}",
            //     destructured_patterns.iter().map(
            //         |(id, _, constructor)| (id, constructor.constructor.to_string())
            //     ).collect::<Vec<_>>(),
            // );

            // println!(
            //     "{:?}",
            //     branches_without_overlap.iter().map(
            //         |(condition, (arms, _))| (condition.to_string(), arms.iter().map(|(id, _)| *id).collect::<Vec<_>>())
            //     ).collect::<Vec<_>>(),
            // );

            let mut branches = Vec::with_capacity(branches_without_overlap.len());

            for (condition, (arms, name_bindings)) in branches_without_overlap.into_iter() {
                let node = build_tree(
                    tree_id,
                    &matrix[1..],
                    &arms,
                    session,
                );

                branches.push(DecisionTreeBranch {
                    condition,
                    guard: None,
                    node,
                    name_bindings,
                });
            }

            *tree_id += 1;
            DecisionTreeNode::Tree(DecisionTree {
                id: *tree_id,
                field: Some(matrix[0].field.clone()),
                branches,
            })
        },
        MatrixConstructor::ListSubMatrix(r#type) => {
            let mut indexes = HashSet::new();
            let mut name_bindings = vec![];

            for (_, _, constructor, name_bindings_) in destructured_patterns.into_iter() {
                name_bindings.extend(name_bindings_);

                match &constructor {
                    PatternConstructor::ListSubMatrix { elements, rest } => {
                        let rest_index = match rest {
                            Some(rest) => rest.index,
                            None => elements.len(),
                        };

                        for i in 0..rest_index {
                            indexes.insert(i as i32);
                        }

                        for i in 0..(elements.len() - rest_index) {
                            indexes.insert(-(i as i32 + 1));
                        }
                    },
                    PatternConstructor::Wildcard => {},
                    _ => todo!(),
                }
            }

            let mut indexes = indexes.into_iter().collect::<Vec<_>>();
            indexes.sort();
            let sub_matrix = get_list_sub_matrix(r#type, &matrix[0].field, &indexes, session);
            let new_matrix = vec![
                sub_matrix,
                matrix[1..].to_vec()
            ].concat();

            *tree_id += 1;
            DecisionTreeNode::Tree(DecisionTree {
                id: *tree_id,
                field: Some(matrix[0].field.clone()),

                // no branches
                branches: vec![DecisionTreeBranch {
                    condition: ExprConstructor::Wildcard,
                    guard: None,
                    node: build_tree(
                        tree_id,
                        &new_matrix,
                        arms,
                        session,
                    ),
                    name_bindings,
                }],
            })
        },
    }
}

fn split_or_patterns(
    destructured_patterns: Vec<(usize, &MatchArm, PatternConstructor, Vec<NameBinding>)>,
) -> Vec<(usize, &MatchArm, PatternConstructor, Vec<NameBinding>)> {
    let mut result = Vec::with_capacity(destructured_patterns.len());

    for (id, arm, pattern, name_bindings) in destructured_patterns.into_iter() {
        match pattern {
            PatternConstructor::Or(patterns) => {
                for pattern in patterns.into_iter() {
                    result.push((id, arm, pattern, name_bindings.clone()));
                }
            },
            pattern => {
                result.push((id, arm, pattern, name_bindings));
            },
        }
    }

    result
}

fn true_value(session: &Session) -> Expr {
    Expr::Ident(IdentWithOrigin {
        id: intern_string(b"True", &session.intermediate_dir).unwrap(),
        span: Span::None,
        origin: NameOrigin::Foreign {
            kind: NameKind::EnumVariant {
                parent: session.get_lang_item_span("type.Bool"),
            },
        },
        def_span: session.get_lang_item_span("variant.Bool.True"),
    })
}

fn false_value(session: &Session) -> Expr {
    Expr::Ident(IdentWithOrigin {
        id: intern_string(b"False", &session.intermediate_dir).unwrap(),
        span: Span::None,
        origin: NameOrigin::Foreign {
            kind: NameKind::EnumVariant {
                parent: session.get_lang_item_span("type.Bool"),
            },
        },
        def_span: session.get_lang_item_span("variant.Bool.False"),
    })
}

#[derive(Clone, Debug)]
pub enum ExprConstructor {
    Range(Range),
    Or(Vec<ExprConstructor>),
    Wildcard,
}

impl fmt::Display for ExprConstructor {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            ExprConstructor::Range(r) => format!("{r}"),
            ExprConstructor::Or(cs) => format!("{}", cs.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" | ")),
            ExprConstructor::Wildcard => String::from("_"),
        };

        write!(fmt, "{s}")
    }
}
