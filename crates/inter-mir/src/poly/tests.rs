use super::PolySolver;
use sodigy_mir::Type;
use sodigy_span::{PolySpanKind, Span};
use sodigy_string::InternedString;
use std::collections::{HashMap, HashSet};

// TODO: It's too difficult to debug.
//       How about making `PolySolver` generic?
#[test]
fn poly_solvers() {
    for (impls, cases) in [
        (
            vec![
                (poly_impl(1), vec![int(), int()]),
                (poly_impl(2), vec![int(), number()]),
                (poly_impl(3), vec![tuple2(int(), int()), generic_param_t()]),
            ],
            vec![
                (0, vec![int(), int()], Some(poly_impl(1))),
                (1, vec![int(), number()], Some(poly_impl(2))),
                (2, vec![number(), int()], None),
                (3, vec![tuple2(int(), int()), int()], Some(poly_impl(3))),
            ],
        ),
        (
            vec![
                (poly_impl(1), vec![generic_param_t(), generic_param_t()]),
                (poly_impl(2), vec![int(), number()]),
            ],
            vec![
                (4, vec![type_var_x(), type_var_x()], Some(poly_impl(1))),
                (5, vec![int(), int()], Some(poly_impl(1))),
                (6, vec![number(), number()], Some(poly_impl(1))),
                (7, vec![int(), number()], Some(poly_impl(2))),
                (8, vec![number(), int()], None),
            ],
        ),
    ] {
        let mut solver = PolySolver::new();

        for (impl_id, types) in impls.into_iter() {
            let types = into_generics(types);
            solver.impls.insert(impl_id, types);
        }

        solver.build_state_machine();

        for (case_id, types, answer) in cases.into_iter() {
            println!("{case_id}");
            let types = into_generics(types);

            match &solver.state_machine {
                Some(state_machine) => {
                    let candidates = state_machine.get_candidates(&types);
                    assert_no_duplicate(candidates);
                    check_answer(candidates, answer);
                },
                None => unreachable!(),
            }
        }
    }
}

fn assert_no_duplicate(candidates: &[Span]) {
    assert_eq!(
        candidates.len(),
        candidates.iter().map(|s| *s).collect::<HashSet<_>>().len(),
    );
}

fn check_answer(candidates: &[Span], answer: Option<Span>) {
    match answer {
        Some(a) => assert!(candidates.contains(&a)),
        None => assert!(candidates.is_empty()),
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

fn generic_params() -> Vec<Span> {
    (50000..50010).map(|n| dummy_span(n)).collect()
}

fn into_generics(types: Vec<Type>) -> HashMap<Span, Type> {
    types.into_iter().zip(generic_params().into_iter()).map(
        |(r#type, generic)| (generic, r#type)
    ).collect()
}
