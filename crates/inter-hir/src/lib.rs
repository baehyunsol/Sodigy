use sodigy_hir::{Expr, FuncArgDef, StructField, Type};
use sodigy_name_analysis::NameKind;
use sodigy_span::Span;
use std::collections::HashMap;

mod endec;
mod session;

pub use session::Session;

impl Session {
    pub fn ingest(
        &mut self,
        module_span: Span,  // of this hir
        hir_session: sodigy_hir::Session,
    ) {
        for (def_span, (args, generics)) in hir_session.funcs.iter().map(
            |func| (
                func.name_span,
                (
                    func.args.iter().map(
                        |arg| FuncArgDef {
                            name: arg.name,
                            name_span: arg.name_span,
                            r#type: None,
                            default_value: arg.default_value,
                        }
                    ).collect(),
                    func.generics.clone(),
                ),
            )
        ) {
            self.func_shapes.insert(def_span, (args, generics));
        }

        for (def_span, (fields, generics)) in hir_session.structs.iter().map(
            |r#struct| (
                r#struct.name_span,
                (
                    r#struct.fields.iter().map(
                        |field| StructField {
                            name: field.name,
                            name_span: field.name_span,
                            r#type: None,
                            default_value: field.default_value,
                        }
                    ).collect(),
                    r#struct.generics.clone(),
                ),
            )
        ) {
            self.struct_shapes.insert(def_span, (fields, generics));
        }

        let mut children = HashMap::new();

        for (name, span, _) in hir_session.iter_public_names() {
            children.insert(name, span);
        }

        self.module_name_map.insert(
            module_span,
            (
                module_span,
                NameKind::Module,
                children,
            ),
        );
    }

    pub fn resolve(&mut self, hir_session: &mut sodigy_hir::Session) {
        self.name_aliases = HashMap::new();
        self.type_aliases = HashMap::new();

        for r#use in hir_session.uses.iter() {
            self.name_aliases.insert(r#use.name_span, r#use.clone());
        }

        for alias in hir_session.aliases.iter() {
            self.type_aliases.insert(alias.name_span, alias.clone());
        }

        self.resolve_alias_recursive();

        if !self.errors.is_empty() {
            return;
        }

        for r#let in hir_session.lets.iter_mut() {
            if let Some(r#type) = &mut r#let.r#type {
                self.resolve_type_recursive(r#type);
            }

            self.resolve_expr_recursive(&mut r#let.value);
        }
    }

    // If there's `use x as y;` and `use y as z;`, we have to
    // replace `use y as z;` with `use x as z;`.
    // Also, if there's `type MyInt = Int;` and `type YourInt = MyInt;`,
    // we have to replace `type YourInt = MyInt;` with `type YourInt = Int;`.
    pub fn resolve_alias_recursive(&mut self) {
        // TODO: make recursion limit configurable (it's hard-coded to 32)
        for i in 0..33 {
            let mut nested_aliases = HashMap::new();

            for (def_span, name_alias) in self.name_aliases.iter() {
                if let Some(new_alias) = self.name_aliases.get(&name_alias.root.def_span) {
                    nested_aliases.insert(*def_span, new_alias.clone());
                }
            }

            if i == 32 {
                self.errors.push();
            }

            else if !nested_aliases.is_empty() {
                // TODO: apply aliases
                todo!()
            }

            else {
                break;
            }
        }

        for i in 0..33 {
            let mut nested_aliases = HashMap::new();

            for (def_span, type_alias) in self.type_aliases.iter() {
                match &type_alias.r#type {
                    Type::Identifier(id) | Path { id, .. } => todo!(),
                    Type::Param { r#type, .. } => todo!(),
                    Type::Tuple { types, .. } => todo!(),
                    Type::Func { args, r#return, .. } => todo!(),
                    Type::Wildcard(_) => {},
                }
            }

            if i == 32 {
                self.errors.push();
            }

            else if !nested_aliases.is_empty() {
                // TODO: apply aliases
                todo!()
            }

            else {
                break;
            }
        }
    }

    pub fn resolve_type_recursive(&mut self, r#type: &mut Type) {
        todo!()
    }

    pub fn resolve_expr_recursive(&mut self, expr: &mut Expr) {
        todo!()
    }
}
