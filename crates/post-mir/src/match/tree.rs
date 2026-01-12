use super::{
    Constructor,
    LiteralType,
    NameBinding,
    Range,
    merge_conditions,
    read_field_of_pattern,
    remove_overlaps,
};
use sodigy_error::{Error, Warning};
use sodigy_hir::LetOrigin;
use sodigy_mir::{Block, Callable, Expr, If, Let, MatchArm};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_parse::Field;
use sodigy_span::{Span, SpanDeriveKind};
use sodigy_string::intern_string;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

// In this tree, it reads `scrutinee.field` and branches to the next tree (or leaf).
// `.condition` of each branch must be non-overlapping.
//
// `field` is None if it doesn't have to check scrutinee (e.g. when the branch is
// based on the match guards.
#[derive(Clone, Debug)]
pub struct DecisionTree {
    // It's used for spans. Just make sure that trees in a match expression have unique id.
    pub id: u32,

    pub field: Option<Vec<Field>>,
    pub branches: Vec<DecisionTreeBranch>,
}

impl DecisionTree {
    /// ```ignore
    /// match (x, y) {
    ///     (0, 0) => 0,
    ///     (0, a) => a,
    ///     (1, 1) => 2,
    ///     (a, _) => a,
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
        lang_items: &HashMap<String, Span>,
        intermediate_dir: &str,
    ) -> Expr {
        // TODO: We need some kinda cache for the scrutinee.
        //       For example, if there's `let curr = scrutinee._0._0` and `let curr = scrutinee._0._0._1`,
        //       we don't have to evaluate `scrutinee._0._0` twice.
        let curr_field_name = intern_string(b"curr", "").unwrap();
        let curr_field_span = scrutinee.error_span_wide().derive(SpanDeriveKind::MatchScrutinee(self.id));
        let curr_field = Expr::Ident(IdentWithOrigin {
            id: curr_field_name,
            span: curr_field_span,
            def_span: curr_field_span,
            origin: NameOrigin::Local {
                kind: NameKind::Let { is_top_level: false },
            },
        });
        let mut lets = match &self.field {
            Some(field) => vec![Let {
                keyword_span: Span::None,
                name: curr_field_name,
                name_span: curr_field_span,
                type_annot_span: None,
                value: Expr::Path {
                    lhs: Box::new(scrutinee.clone()),
                    fields: field.clone(),
                },
                origin: LetOrigin::Match,
            }],
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

                lets.push(Let {
                    keyword_span: Span::None,
                    name: name_binding.name,
                    name_span: name_binding.name_span,
                    type_annot_span: None,
                    value: Expr::Ident(IdentWithOrigin {
                        id: curr_field_name,
                        span: Span::None,
                        def_span: curr_field_span,
                        origin: NameOrigin::Local {
                            kind: NameKind::Let { is_top_level: false },
                        },
                    }),
                    origin: LetOrigin::Match,
                });
                name_binding_spans.insert(name_binding.name_span);
            }
        }

        let value = branches_to_expr(
            &self.branches,
            scrutinee,
            &curr_field,
            arms,
            lang_items,
            intermediate_dir,
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
                Constructor::Or(cs) => cs.len(),
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
    pub condition: Constructor,
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
    lang_items: &HashMap<String, Span>,
    intermediate_dir: &str,
) -> Expr {
    match branches {
        [branch] => match &branch.node {
            DecisionTreeNode::Tree(tree) => tree.into_expr(scrutinee, arms, lang_items, intermediate_dir),
            DecisionTreeNode::Leaf { matched, .. } => arms[*matched].1.value.clone(),
        },
        branches => Expr::If(If {
            if_span: Span::None,
            cond: Box::new(branch_condition_to_expr(&branches[0], curr_field, lang_items, intermediate_dir)),
            else_span: Span::None,
            true_value: Box::new(branches_to_expr(&branches[0..1], scrutinee, curr_field, arms, lang_items, intermediate_dir)),
            true_group_span: Span::None,
            false_value: Box::new(branches_to_expr(&branches[1..], scrutinee, curr_field, arms, lang_items, intermediate_dir)),
            false_group_span: Span::None,
            from_short_circuit: None,
        }),
    }
}

fn branch_condition_to_expr(
    branch: &DecisionTreeBranch,
    curr_field: &Expr,
    lang_items: &HashMap<String, Span>,
    intermediate_dir: &str,
) -> Expr {
    match &branch.condition {
        Constructor::Wildcard => match &branch.guard {
            Some(guard) => guard.clone(),
            None => true_value(lang_items, intermediate_dir),
        },
        c => constructor_to_expr(c, curr_field, lang_items, intermediate_dir),
    }
}

fn constructor_to_expr(
    constructor: &Constructor,
    curr_field: &Expr,
    lang_items: &HashMap<String, Span>,
    intermediate_dir: &str,
) -> Expr {
    match constructor {
        Constructor::Tuple(_) => true_value(lang_items, intermediate_dir),
        Constructor::DefSpan(_) => todo!(),
        Constructor::Range(range) => {
            let (lang_item, operand) = match (&range.lhs, &range.rhs) {
                (Some(lhs), Some(rhs)) => match lhs.cmp(rhs) {
                    Ordering::Equal if range.lhs_inclusive && range.rhs_inclusive => match range.r#type {
                        LiteralType::Int => ("built_in.eq_int", Expr::Number { n: lhs.clone(), span: Span::None }),
                        _ => todo!(),
                    },
                    Ordering::Less => {
                        if range.r#type.is_int_like() && &lhs.add_one() == rhs {
                            // `3 < x && x <= 4` is just `x == 4`
                            match (range.r#type, range.lhs_inclusive, range.rhs_inclusive) {
                                (LiteralType::Int, false, true) => ("built_in.eq_int", Expr::Number { n: rhs.clone(), span: Span::None }),
                                (LiteralType::Int, true, false) => ("built_in.eq_int", Expr::Number { n: lhs.clone(), span: Span::None }),
                                (LiteralType::Int, _, _) => {
                                    return true_value(lang_items, intermediate_dir);
                                },
                                // we need `built_in.eq_scalar`
                                _ => todo!(),
                            }
                        }

                        // `0..10` is lowered to `0 <= x && x < 10`, which is then lowered to
                        // `if 0 <= x { x < 10 } else { False }`
                        else {
                            let (lhs, rhs) = match range.r#type {
                                LiteralType::Int => (
                                    Expr::Number { n: lhs.clone(), span: Span::None },
                                    Expr::Number { n: rhs.clone(), span: Span::None },
                                ),
                                _ => todo!(),
                            };
                            let f1 = if range.lhs_inclusive {
                                "fn.leq_int"
                            } else {
                                "built_in.lt_int"
                            };
                            let f2 = if range.rhs_inclusive {
                                "fn.leq_int"
                            } else {
                                "built_in.lt_int"
                            };

                            return Expr::If(If {
                                if_span: Span::None,
                                cond: Box::new(Expr::Call {
                                    func: Callable::Static {
                                        def_span: *lang_items.get(f1).unwrap(),
                                        span: Span::None,
                                    },
                                    args: vec![
                                        lhs,
                                        curr_field.clone(),
                                    ],
                                    arg_group_span: Span::None,
                                    // Poly-solving is already done, so we don't need this.
                                    generic_defs: vec![],
                                    given_keyword_arguments: vec![],
                                }),
                                else_span: Span::None,
                                true_value: Box::new(Expr::Call {
                                    func: Callable::Static {
                                        def_span: *lang_items.get(f2).unwrap(),
                                        span: Span::None,
                                    },
                                    args: vec![
                                        curr_field.clone(),
                                        rhs,
                                    ],
                                    arg_group_span: Span::None,
                                    // Poly-solving is already done, so we don't need this.
                                    generic_defs: vec![],
                                    given_keyword_arguments: vec![],
                                }),
                                true_group_span: Span::None,
                                false_value: Box::new(Expr::Ident(IdentWithOrigin {
                                    id: intern_string(b"False", intermediate_dir).unwrap(),
                                    span: Span::None,
                                    origin: NameOrigin::Foreign {
                                        kind: NameKind::EnumVariant {
                                            parent: *lang_items.get("type.Bool").unwrap(),
                                        },
                                    },
                                    def_span: *lang_items.get("variant.Bool.False").unwrap(),
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
                        return false_value(lang_items, intermediate_dir);
                    },
                },
                (Some(lhs), None) => match (range.r#type, range.lhs_inclusive) {
                    (LiteralType::Int, true) => ("built_in.geq_int", Expr::Number { n: lhs.clone(), span: Span::None }),
                    (LiteralType::Int, false) => ("built_in.gt_int", Expr::Number { n: lhs.clone(), span: Span::None }),
                    _ => todo!(),
                },
                (None, Some(rhs)) => match (range.r#type, range.rhs_inclusive) {
                    (LiteralType::Int, true) => ("built_in.leq_int", Expr::Number { n: rhs.clone(), span: Span::None }),
                    (LiteralType::Int, false) => ("built_in.lt_int", Expr::Number { n: rhs.clone(), span: Span::None }),
                    _ => todo!(),
                },
                (None, None) => {
                    return true_value(lang_items, intermediate_dir);
                },
            };

            Expr::Call {
                func: Callable::Static {
                    def_span: *lang_items.get(lang_item).unwrap(),
                    span: Span::None,
                },
                args: vec![
                    curr_field.clone(),
                    operand,
                ],
                arg_group_span: Span::None,
                // Poly-solving is already done, so we don't need this.
                generic_defs: vec![],
                given_keyword_arguments: vec![],
            }
        },
        Constructor::Or(cs) => constructors_to_expr(cs, curr_field, lang_items, intermediate_dir),
        Constructor::Wildcard => true_value(lang_items, intermediate_dir),
    }
}

fn constructors_to_expr(
    constructors: &[Constructor],
    curr_field: &Expr,
    lang_items: &HashMap<String, Span>,
    intermediate_dir: &str,
) -> Expr {
    match constructors.len() {
        1 => constructor_to_expr(&constructors[0], curr_field, lang_items, intermediate_dir),
        _ => Expr::If(If {
            if_span: Span::None,
            cond: Box::new(constructor_to_expr(&constructors[0], curr_field, lang_items, intermediate_dir,)),
            else_span: Span::None,
            true_value: Box::new(Expr::Ident(IdentWithOrigin {
                id: intern_string(b"True", intermediate_dir).unwrap(),
                span: Span::None,
                origin: NameOrigin::Foreign {
                    kind: NameKind::EnumVariant {
                        parent: *lang_items.get("type.Bool").unwrap(),
                    },
                },
                def_span: *lang_items.get("variant.Bool.True").unwrap(),
            })),
            true_group_span: Span::None,
            false_value: Box::new(constructors_to_expr(&constructors[1..], curr_field, lang_items, intermediate_dir)),
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
    matrix: &[(Vec<Field>, Constructor)],
    arms: &[(usize, &MatchArm)],
    errors: &mut Vec<Error>,
    warnings: &mut Vec<Warning>,
) -> Result<DecisionTreeNode, ()> {
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
                    return Ok(DecisionTreeNode::Leaf {
                        matched: *id,
                        unmatched: unmatched_ids,
                    });
                },
            },
            _ => {
                *tree_id += 1;
                return Ok(DecisionTreeNode::Tree(DecisionTree {
                    id: *tree_id,
                    field: None,
                    branches: branches.into_iter().map(
                        |(guard, id)| DecisionTreeBranch {
                            condition: Constructor::Wildcard,
                            guard,
                            node: DecisionTreeNode::Leaf {
                                matched: id,
                                unmatched: unmatched_ids.clone(),
                            },
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

            *tree_id += 1;
            Ok(DecisionTreeNode::Tree(DecisionTree {
                id: *tree_id,
                field: Some(matrix[0].0.clone()),

                // no branches
                branches: vec![DecisionTreeBranch {
                    condition: Constructor::Wildcard,
                    guard: None,
                    node: build_tree(
                        tree_id,
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
            let mut branches_with_overlap: Vec<(Range, (Vec<(usize, &MatchArm)>, Vec<NameBinding>))> = vec![];

            // default: wildcard
            branches_with_overlap.push((
                Range {
                    r#type: *r#type,
                    lhs: None,
                    lhs_inclusive: false,
                    rhs: None,
                    rhs_inclusive: false,
                },
                (vec![], vec![]),
            ));

            for (id, arm, pattern) in destructured_patterns.iter() {
                match &pattern.constructor {
                    Constructor::Range(r) => {
                        if r.r#type != *r#type {
                            errors.push(Error::todo(19200, "type errors in patterns", pattern.pattern.error_span_wide()));
                        }

                        else {
                            let mut is_new = true;

                            for (br, (arms, name_bindings)) in branches_with_overlap.iter_mut() {
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

                                branches_with_overlap.push((r.clone(), (vec![(*id, *arm)], name_bindings)));
                            }
                        }
                    },
                    Constructor::Wildcard => {
                        for (_, (arms, name_bindings)) in branches_with_overlap.iter_mut() {
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

            let branches_without_overlap = remove_overlaps(branches_with_overlap);
            let branches_without_overlap = merge_conditions(branches_without_overlap);
            let mut branches = Vec::with_capacity(branches_without_overlap.len());
            let mut has_error = false;

            for (condition, (arms, name_bindings)) in branches_without_overlap.into_iter() {
                match build_tree(
                    tree_id,
                    &matrix[1..],
                    &arms,
                    errors,
                    warnings,
                ) {
                    Ok(node) => {
                        branches.push(DecisionTreeBranch {
                            condition,
                            guard: None,
                            node,
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
                *tree_id += 1;
                Ok(DecisionTreeNode::Tree(DecisionTree {
                    id: *tree_id,
                    field: Some(matrix[0].0.clone()),
                    branches,
                }))
            }
        },
        c => panic!("TODO: {c:?}"),
    }
}

fn true_value(lang_items: &HashMap<String, Span>, intermediate_dir: &str) -> Expr {
    Expr::Ident(IdentWithOrigin {
        id: intern_string(b"True", intermediate_dir).unwrap(),
        span: Span::None,
        origin: NameOrigin::Foreign {
            kind: NameKind::EnumVariant {
                parent: *lang_items.get("type.Bool").unwrap(),
            },
        },
        def_span: *lang_items.get("variant.Bool.True").unwrap(),
    })
}

fn false_value(lang_items: &HashMap<String, Span>, intermediate_dir: &str) -> Expr {
    Expr::Ident(IdentWithOrigin {
        id: intern_string(b"False", intermediate_dir).unwrap(),
        span: Span::None,
        origin: NameOrigin::Foreign {
            kind: NameKind::EnumVariant {
                parent: *lang_items.get("type.Bool").unwrap(),
            },
        },
        def_span: *lang_items.get("variant.Bool.False").unwrap(),
    })
}
