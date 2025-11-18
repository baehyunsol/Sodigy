use crate::{GenericCall, Solver};
use sodigy_hir::Poly;
use sodigy_mir::{Session, Type};
use sodigy_span::Span;
use std::collections::HashMap;

impl Solver {
    pub fn try_solve_poly(
        &mut self,
        polys: &HashMap<Span, Poly>,
        solvers: &HashMap<Span, PolySolver>,
        generic_call: &GenericCall,
    ) -> Result<SolvePolyResult, ()> {
        match polys.get(&generic_call.def) {
            Some(poly) => {
                let solver = solvers.get(&poly.name_span).unwrap();
                let candidates = solver.solve(&generic_call.generics);

                match candidates.len() {
                    0 => {
                        if poly.has_default_impl {
                            Ok(SolvePolyResult::DefaultImpl(poly.name_span))
                        }

                        else {
                            Ok(SolvePolyResult::NoCandidates)
                        }
                    },
                    1 => Ok(SolvePolyResult::OneCandidate(candidates[0])),
                    2.. => Ok(SolvePolyResult::MultiCandidates(candidates)),
                }
            },
            None => Ok(SolvePolyResult::NotPoly),
        }
    }

    pub fn init_poly_solvers(&self, session: &Session) -> Result<HashMap<Span, PolySolver>, ()> {
        let mut result = HashMap::new();

        for (span, poly) in session.polys.iter() {
            match poly.impls.len() {
                0 => {
                    result.insert(*span, PolySolver::never());
                },
                1 => todo!(),
                2.. => todo!(),
            }
        }

        Ok(result)
    }
}

pub struct PolySolver;

impl PolySolver {
    // a solver that always returns `vec![]`
    pub fn never() -> PolySolver {
        todo!()
    }

    pub fn solve(&self, generics: &HashMap<Span, Type>) -> Vec<Span> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub enum SolvePolyResult {
    NotPoly,
    DefaultImpl(Span),
    NoCandidates,
    OneCandidate(Span),
    MultiCandidates(Vec<Span>),
}
