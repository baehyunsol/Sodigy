use super::{
    Constructor,
    NameBinding,
    Range,
    read_field_of_pattern,
};
use sodigy_error::{Error, Warning};
use sodigy_mir::{Block, Expr, Let, MatchArm};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_parse::Field;
use sodigy_span::Span;
use sodigy_string::intern_string;

// In this state, it reads `scrutinee.field` and transits to the next state.
// There must be exactly 1 `transition` whose `.condition` meets `scrutinee.field`.
// If there are more than 1 transition, that's an ICE.
//
// `field` is None if it doesn't have to check scrutinee (e.g. when the transition is
// based on the match guards.
#[derive(Clone, Debug)]
pub struct StateMachine {
    pub field: Option<Vec<Field>>,
    pub transitions: Vec<Transition>,
}

impl StateMachine {
    /// ```
    /// match x {
    ///     (0, 0) => 0,
    ///     (0, a) => a,
    ///     (1, 1) => 2,
    ///     (a, _) => a,
    /// }
    /// ```
    /// ->
    /// ```
    /// {
    ///     let curr = x._0;
    ///     let a = curr;
    ///
    ///     if curr == 0 {
    ///         let curr = x._1;
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
    pub fn into_expr(&self, arms: &[(usize, &MatchArm)]) -> Expr {
        let current_field = (intern_string(b"curr", "").unwrap(), Span::None);
        let mut lets = match &self.field {
            Some(field) => vec![Let {
                name: current_field.0,
                name_span: current_field.1,
                type_annot_span: None,
                value: todo!(),
            }],
            None => vec![],
        };

        for Transition { name_bindings, .. } in self.transitions.iter() {
            for name_binding in name_bindings.iter() {
                lets.push(Let {
                    name: name_binding.name,
                    name_span: name_binding.name_span,
                    type_annot_span: None,
                    value: Expr::Ident(IdentWithOrigin {
                        id: current_field.0,
                        span: Span::None,
                        def_span: current_field.1,
                        origin: NameOrigin::Local {
                            kind: NameKind::Let { is_top_level: false },
                        },
                    }),
                });
            }
        }

        let value = match &self.transitions[..] {
            [transition] => match &transition.state {
                StateMachineOrArm::StateMachine(fsm) => fsm.into_expr(arms),
                StateMachineOrArm::Arm { matched, .. } => arms[*matched].1.value.clone(),
            },
            _ => todo!(),
        };

        Expr::Block(Block {
            group_span: Span::None,
            lets,
            asserts: vec![],
            value: Box::new(value),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Transition {
    pub condition: Constructor,
    pub guard: Option<Expr>,
    pub state: StateMachineOrArm,

    // If the condition is met, `scrutinee.field` is bound to the name.
    // It's bound AFTER `scrutinee.field` is evaluated and BEFORE the transition.
    pub name_bindings: Vec<NameBinding>,
}

#[derive(Clone, Debug)]
pub enum StateMachineOrArm {
    StateMachine(StateMachine),

    // If there are multiple arms that can reach here, the first
    // arm is matched, and the remainings are pushed to `unmatched`.
    // `unmatched` is later used for warning messages, if necessary.
    Arm {
        matched: usize,
        unmatched: Vec<usize>,
    },
}

pub(crate) fn build_state_machine(
    matrix: &[(Vec<Field>, Constructor)],
    arms: &[(usize, &MatchArm)],
    errors: &mut Vec<Error>,
    warnings: &mut Vec<Warning>,
) -> Result<StateMachineOrArm, ()> {
    if matrix.is_empty() {
        let mut transitions = vec![];
        let mut unmatched_ids = vec![];

        for (i, (id, arm)) in arms.iter().enumerate() {
            if let Some(guard) = &arm.guard {
                transitions.push((Some(guard.clone()), *id));
            }

            else {
                transitions.push((None, *id));

                if i + 1 < arms.len() {
                    unmatched_ids = arms[(i + 1)..].iter().map(|(id, _)| *id).collect();
                }

                break;
            }
        }

        match transitions.len() {
            0 => todo!(),
            1 => match &transitions[0] {
                (Some(guard), _) => todo!(),
                (None, id) => {
                    return Ok(StateMachineOrArm::Arm {
                        matched: *id,
                        unmatched: unmatched_ids,
                    });
                },
            },
            _ => {
                return Ok(StateMachineOrArm::StateMachine(StateMachine {
                    field: None,
                    transitions: transitions.into_iter().map(
                        |(guard, id)| Transition {
                            condition: Constructor::Wildcard,
                            guard,
                            state: StateMachineOrArm::Arm {
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
