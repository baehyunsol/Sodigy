use super::PolySolver;
use crate::RenderStateMachine;
use sodigy_mir::Type;
use sodigy_span::{PolySpanKind, Span};
use sodigy_string::InternedString;
use std::collections::{HashMap, HashSet};

#[test]
fn poly_solvers() {
    let name_map: HashMap<Span, String> = (0..10).map(
        |i| (poly_impl(i), format!("poly-impl-{i}"))
    ).collect();
    let session = TestSession::new();

    for (impls, cases) in [
        (
            vec![
                (poly_impl(1), vec![int(), int()]),
                (poly_impl(2), vec![int(), number()]),
                (poly_impl(3), vec![tuple2(int(), int()), generic_param_t()]),
            ],
            vec![
                (0, vec![int(), int()], vec![poly_impl(1)], false),
                (1, vec![int(), number()], vec![poly_impl(2)], false),
                (2, vec![number(), int()], vec![], false),
                (3, vec![tuple2(int(), int()), int()], vec![poly_impl(3)], false),
                (4, vec![type_var_x(), type_var_y()], vec![poly_impl(1), poly_impl(2), poly_impl(3)], false),
            ],
        ),
        (
            vec![
                (poly_impl(1), vec![generic_param_t(), generic_param_t()]),
                (poly_impl(2), vec![int(), number()]),
            ],
            vec![
                // Current implementation is not strong enough to filter out poly_impl(2) in this case.
                (10, vec![type_var_x(), type_var_x()], vec![poly_impl(1)], true),

                (11, vec![int(), int()], vec![poly_impl(1)], false),
                (12, vec![number(), number()], vec![poly_impl(1)], false),
                (13, vec![int(), number()], vec![poly_impl(2)], false),

                // Current implementation is not strong enough to filter out poly_impl(1) in this case.
                (14, vec![number(), int()], vec![], true),
            ],
        ),
        (
            vec![
                (poly_impl(1), vec![int()]),
                (poly_impl(2), vec![number()]),
            ],
            vec![
                (20, vec![int()], vec![poly_impl(1)], false),
                (21, vec![number()], vec![poly_impl(2)], false),
                (22, vec![type_var_x()], vec![poly_impl(1), poly_impl(2)], false),
            ],
        ),
        (
            vec![
                (poly_impl(1), vec![generic_param_t(), generic_param_u()]),
                (poly_impl(2), vec![int(), number()]),
            ],
            vec![
                (30, vec![int(), number()], vec![poly_impl(1), poly_impl(2)], false),
                (31, vec![type_var_x(), type_var_y()], vec![poly_impl(1), poly_impl(2)], false),
            ],
        ),
        (
            vec![
                (poly_impl(1), vec![generic_param_t(), byte()]),
                (poly_impl(2), vec![int(), int()]),
                (poly_impl(3), vec![byte(), number()]),
            ],
            vec![
                (40, vec![type_var_x(), type_var_y()], vec![poly_impl(1), poly_impl(2), poly_impl(3)], false),
                (41, vec![int(), int()], vec![poly_impl(2)], false),
                (42, vec![int(), byte()], vec![poly_impl(1)], false),
                (43, vec![int(), type_var_x()], vec![poly_impl(1), poly_impl(2)], false),
            ],
        ),
        (
            vec![
                (poly_impl(1), vec![generic_param_t(), generic_param_t()]),
                (poly_impl(2), vec![int(), byte()]),
                (poly_impl(3), vec![int(), number()]),
                (poly_impl(4), vec![int(), tuple2(int(), int())]),
                (poly_impl(5), vec![byte(), number()]),
                (poly_impl(6), vec![byte(), int()]),
                (poly_impl(7), vec![number(), int()]),
            ],
            vec![
                (50, vec![int(), int()], vec![poly_impl(1)], false),
            ],
        ),
        (
            vec![
                (poly_impl(1), vec![generic_param_t(), generic_param_t(), generic_param_u()]),
                (poly_impl(2), vec![generic_param_t(), generic_param_u(), generic_param_t()]),
                (poly_impl(3), vec![generic_param_u(), generic_param_t(), generic_param_t()]),
                (poly_impl(4), vec![generic_param_t(), generic_param_t(), generic_param_t()]),
                (poly_impl(5), vec![int(), byte(), number()]),
            ],
            vec![
                (60, vec![int(), int(), number()], vec![poly_impl(1)], true),
                (61, vec![byte(), number(), byte()], vec![poly_impl(2)], true),
            ],
        ),
    ] {
        let mut solver = PolySolver::new();

        for (impl_id, types) in impls.into_iter() {
            let types = into_generics(types);
            solver.impls.insert(impl_id, types);
        }

        solver.build_state_machine();
        println!("---------");
        if let Some(state_machine) = &solver.state_machine {
            println!("{}", session.render_state_machine_inner(state_machine, &name_map, 1));
        } else {
            println!("/* No State Machine */");
        }
        println!("---------");

        for (case_id, types, answer, false_positive) in cases.into_iter() {
            println!("case: {case_id}");

            let types = into_generics(types);

            match &solver.state_machine {
                Some(state_machine) => {
                    let candidates = state_machine.get_candidates(&types);
                    let candidates = candidates.iter().map(
                        |s| name_map.get(s).unwrap().to_string()
                    ).collect::<Vec<_>>();
                    let answer = answer.iter().map(
                        |s| name_map.get(s).unwrap().to_string()
                    ).collect::<Vec<_>>();

                    assert_no_duplicate(&candidates);
                    check_answer(&candidates, &answer, false_positive);
                },
                None => unreachable!(),
            }
        }
    }
}

fn assert_no_duplicate(candidates: &[String]) {
    assert_eq!(
        candidates.len(),
        candidates.iter().collect::<HashSet<_>>().len(),
    );
}

fn check_answer(candidates: &[String], answer: &[String], false_positive: bool) {
    if false_positive {
        if !answer.iter().all(|c| candidates.contains(c)) {
            panic!("candidates: {candidates:?}, answer: {answer:?}");
        }
    }

    else {
        assert_eq!(
            candidates.iter().collect::<HashSet<_>>(),
            answer.iter().collect::<HashSet<_>>(),
        );
    }
}

fn dummy_span(id: u32) -> Span {
    Span::Poly {
        name: InternedString::dummy(),
        kind: PolySpanKind::Name,
        monomorphize_id: Some(id as u128),
    }
}

fn simple_type(id: u32) -> Type {
    Type::Data {
        constructor_def_span: dummy_span(id + 10000),
        constructor_span: Span::None,
        args: None,
        group_span: None,
    }
}

fn type_with_args(id: u32, args: Vec<Type>) -> Type {
    Type::Data {
        constructor_def_span: dummy_span(id + 20000),
        constructor_span: Span::None,
        args: Some(args),
        group_span: Some(Span::None),
    }
}

fn poly_impl(n: u32) -> Span {
    dummy_span(n + 30000)
}

fn int() -> Type {
    simple_type(0)
}

fn number() -> Type {
    simple_type(1)
}

fn byte() -> Type {
    simple_type(2)
}

fn tuple2(a: Type, b: Type) -> Type {
    type_with_args(0, vec![a, b])
}

fn generic_param_t() -> Type {
    Type::GenericParam {
        def_span: dummy_span(40000),
        span: Span::None,
    }
}

fn generic_param_u() -> Type {
    Type::GenericParam {
        def_span: dummy_span(40001),
        span: Span::None,
    }
}

fn type_var_x() -> Type {
    Type::Var {
        def_span: dummy_span(40002),
        is_return: false,
    }
}

fn type_var_y() -> Type {
    Type::Var {
        def_span: dummy_span(40003),
        is_return: false,
    }
}

fn generic_params() -> Vec<Span> {
    (50000..50010).map(|n| dummy_span(n)).collect()
}

fn into_generics(types: Vec<Type>) -> HashMap<Span, Type> {
    types.into_iter().zip(generic_params().into_iter()).map(
        |(r#type, generic)| (generic, r#type)
    ).collect()
}

struct TestSession {}

impl RenderStateMachine for TestSession {
    fn span_to_string_impl(&self, span: Span) -> Option<String> {
        match span {
            Span::Poly {
                monomorphize_id: Some(n),
                ..
            } => match n {
                10000 => Some("int"),
                10001 => Some("number"),
                10002 => Some("byte"),
                20000 => Some("tuple"),
                40000 => Some("T"),
                40001 => Some("U"),
                40002 => Some("X"),
                40003 => Some("Y"),
                50000 => Some("GenericParam(0)"),
                50001 => Some("GenericParam(1)"),
                50002 => Some("GenericParam(2)"),
                50003 => Some("GenericParam(3)"),
                50004 => Some("GenericParam(4)"),
                50005 => Some("GenericParam(5)"),
                _ => None,
            }.map(|s| s.to_string()),
            _ => None,
        }
    }
}

impl TestSession {
    pub fn new() -> Self {
        TestSession {}
    }
}
