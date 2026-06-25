use super::Monomorphization;
use crate::Session;
use sodigy_hir::FuncOrigin;
use sodigy_mir::Func;

impl Session {
    pub fn monomorphize_func(&mut self, func: &Func, monomorphization: &Monomorphization) -> Func {
        let mut params = Vec::with_capacity(func.params.len());

        for param in func.params.iter() {
            let mut new_param = param.clone();
            let old_param_type = self.types.get(&param.name_span).unwrap().clone();
            let new_param_type = self.monomorphize_type(&old_param_type, monomorphization);

            new_param.name_span = new_param.name_span.monomorphize(monomorphization.id);
            self.types.insert(new_param.name_span.clone(), new_param_type);
            params.push(new_param);
        }

        let new_name_span = func.name_span.monomorphize(monomorphization.id);
        let mut new_value = func.value.clone();
        let old_type = self.types.get(&func.name_span).unwrap().clone();
        let new_type = self.monomorphize_type(&old_type, monomorphization);

        self.monomorphize_expr(&mut new_value, monomorphization);
        self.types.insert(new_name_span.clone(), new_type);

        Func {
            is_pure: func.is_pure,
            impure_keyword_span: func.impure_keyword_span.as_ref().map(|span| span.monomorphize(monomorphization.id)),
            keyword_span: func.keyword_span.monomorphize(monomorphization.id),
            name: func.name,
            name_span: new_name_span,
            generics: vec![],
            generic_group_span: None,
            params,
            type_annot_span: func.type_annot_span.as_ref().map(|span| span.monomorphize(monomorphization.id)),
            value: new_value,
            origin: FuncOrigin::Monomorphization,
            built_in: func.built_in,
        }
    }
}
