use super::Monomorphization;
use crate::Session;
use sodigy_hir::FuncOrigin;
use sodigy_mir::Func;

impl Session {
    pub fn monomorphize_func(&mut self, func: &Func, monomorphization: &Monomorphization) -> Func {
        let mut params = Vec::with_capacity(func.params.len());

        for param in func.params.iter() {
            let mut new_param = param.clone();
            new_param.name_span = new_param.name_span.monomorphize(monomorphization.id);
            let new_type = match self.types.get(&param.name_span) {
                Some(r#type) => {
                    let mut r#type = r#type.clone();

                    for (generic_param, generic_arg) in monomorphization.generics.iter() {
                        r#type.substitute_generic_param(generic_param, generic_arg);
                    }

                    r#type
                },
                None => unreachable!(),
            };

            self.types.insert(new_param.name_span.clone(), new_type);
            params.push(new_param);
        }

        let new_name_span = func.name_span.monomorphize(monomorphization.id);
        let new_type = match self.types.get(&func.name_span) {
            Some(r#type) => {
                let mut r#type = r#type.clone();

                for (generic_param, generic_arg) in monomorphization.generics.iter() {
                    r#type.substitute_generic_param(generic_param, generic_arg);
                }

                r#type
            },
            None => unreachable!(),
        };
        let mut new_value = func.value.clone();
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
