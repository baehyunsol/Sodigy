use super::Monomorphization;
use crate::{LogId, Session, write_log};
use sodigy_hir::FuncOrigin;
use sodigy_mir::{Func, Type};
use sodigy_span::Span;

#[cfg(feature = "log")]
use crate::LogEntry;

impl Session {
    pub fn monomorphize_func(
        &mut self,
        func: &Func,
        monomorphization: &Monomorphization,

        // `monomorphize_func` might introduce intermediate_types.
        // For example, when you monomorphize `fn foo<T>(x: Bar<T>)` with `T=[Char]`,
        // you have to monomorphize not only `[Char]`, but also `Bar<[Char]>`.
        intermediate_types: &mut Vec<(Type, Span)>,
    ) -> Func {
        let _id = if cfg!(feature = "log") {
            Some(LogId::new())
        } else {
            None
        };

        write_log!(self, LogEntry::MonomorphizeFuncStart {
            id: _id.unwrap(),
            func: func.clone(),
            monomorphization: monomorphization.clone(),
        });

        let mut params = Vec::with_capacity(func.params.len());

        for param in func.params.iter() {
            let mut new_param = param.clone();
            new_param.name_span = new_param.name_span.monomorphize(monomorphization.id);

            let old_param_type = self.types.get(&param.name_span).unwrap().clone();
            let new_param_type = self.monomorphize_type(&old_param_type, monomorphization);

            if new_param_type.has_to_be_monomorphized() {
                intermediate_types.push((new_param_type.clone(), new_param.name_span.clone()));
            }

            self.types.insert(new_param.name_span.clone(), new_param_type);
            params.push(new_param);
        }

        let new_name_span = func.name_span.monomorphize(monomorphization.id);
        let mut new_value = func.value.clone();
        let old_type = self.types.get(&func.name_span).unwrap().clone();
        let new_type = self.monomorphize_type(&old_type, monomorphization);

        if new_type.has_to_be_monomorphized() {
            intermediate_types.push((new_type.clone(), new_name_span.clone()));
        }

        self.monomorphize_expr(&mut new_value, monomorphization);
        self.types.insert(new_name_span.clone(), new_type);

        let result = Func {
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
        };

        write_log!(self, LogEntry::MonomorphizeFuncEnd {
            id: _id.unwrap(),
            result: result.clone(),
            // TODO: log intermediate_types
        });
        result
    }
}
