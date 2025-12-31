use super::{
    Constructor,
    NameBinding,
    Range,
    read_field_of_pattern,
};
use sodigy_error::{Error, Warning};
use sodigy_hir::LetOrigin;
use sodigy_mir::{Block, Expr, If, Let, MatchArm};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_parse::Field;
use sodigy_span::{Span, SpanDeriveKind};
use sodigy_string::intern_string;

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
    /// ```
    /// match (x, y) {
    ///     (0, 0) => 0,
    ///     (0, a) => a,
    ///     (1, 1) => 2,
    ///     (a, _) => a,
    /// }
    /// ```
    /// ->
    /// ```
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
    pub fn into_expr(&self, scrutinee: &Expr, arms: &[(usize, &MatchArm)]) -> Expr {
        // TODO: We need some kinda cache for the scrutinee.
        //       For example, if there's `let curr = scrutinee._0._0` and `let curr = scrutinee._0._0._1`,
        //       we don't have to evaluate `scrutinee._0._0` twice.
        let curr_field_name = intern_string(b"curr", "").unwrap();
        let curr_field_span = scrutinee.error_span_wide().derive(SpanDeriveKind::MatchScrutinee(self.id));
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

        for DecisionTreeBranch { name_bindings, .. } in self.branches.iter() {
            for name_binding in name_bindings.iter() {
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
            }
        }

        let value = branches_to_expr(&self.branches, scrutinee, arms);

        Expr::Block(Block {
            group_span: Span::None,
            lets,
            asserts: vec![],
            value: Box::new(value),
        })
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
    arms: &[(usize, &MatchArm)],
) -> Expr {
    match branches {
        [branch] => match &branch.node {
            DecisionTreeNode::Tree(tree) => tree.into_expr(scrutinee, arms),
            DecisionTreeNode::Leaf { matched, .. } => arms[*matched].1.value.clone(),
        },
        branches => Expr::If(If {
            if_span: Span::None,
            cond: Box::new(branch_condition_to_expr(&branches[0])),
            else_span: Span::None,
            true_value: Box::new(branches_to_expr(&branches[0..1], scrutinee, arms)),
            true_group_span: Span::None,
            false_value: Box::new(branches_to_expr(&branches[1..], scrutinee, arms)),
            false_group_span: Span::None,
            from_short_circuit: None,
        }),
    }
}

fn branch_condition_to_expr(branch: &DecisionTreeBranch) -> Expr {
    match &branch.condition {
        Constructor::Tuple(_) => unreachable!(),

        // TODO: there's a very big problem
        //       mir-lowering, poly-solving and monomorphizing is already done,
        //       so we cannot use generic functions...
        Constructor::Range(r) => todo!(),
        Constructor::Or(cs) => todo!(),
        Constructor::Wildcard => match &branch.guard {
            Some(guard) => guard.clone(),
            None => unreachable!(),
        },
        _ => todo!(),
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
            let mut branches_with_overlap: Vec<(Range, Vec<(usize, &MatchArm)>, Vec<NameBinding>)> = vec![];

            // default: wildcard
            branches_with_overlap.push((
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

                            for (br, arms, name_bindings) in branches_with_overlap.iter_mut() {
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

                                branches_with_overlap.push((r.clone(), vec![(*id, *arm)], name_bindings));
                            }
                        }
                    },
                    Constructor::Wildcard => {
                        for (_, arms, name_bindings) in branches_with_overlap.iter_mut() {
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

            // TODO: remove overlaps in branches

            let mut branches = Vec::with_capacity(branches_with_overlap.len());
            let mut has_error = false;

            for (range, arms, name_bindings) in branches_with_overlap.into_iter() {
                match build_tree(
                    tree_id,
                    &matrix[1..],
                    &arms,
                    errors,
                    warnings,
                ) {
                    Ok(node) => {
                        branches.push(DecisionTreeBranch {
                            condition: Constructor::Range(range),
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
