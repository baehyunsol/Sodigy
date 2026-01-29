//! After inter-hir is complete, the compiler opens each hir and resolves names
//! in the hir. The entry point is `resolve_module`. The method resolves names
//! in the items, with the methods defined in this module. The methods in this
//! module are all trivial.

use crate::Session;
use sodigy_hir::{
    Assert,
    Func,
    Let,
    Session as HirSession,
    Struct,
};

impl Session {
    pub fn resolve_module(&mut self, hir_session: &mut HirSession) -> Result<(), ()> {
        let mut has_error = false;

        for r#let in hir_session.lets.iter_mut() {
            if let Err(()) = self.resolve_let(r#let) {
                has_error = true;
            }
        }

        for func in hir_session.funcs.iter_mut() {
            if let Err(()) = self.resolve_func(func) {
                has_error = true;
            }
        }

        for r#struct in hir_session.structs.iter_mut() {
            if let Err(()) = self.resolve_struct(r#struct) {
                has_error = true;
            }
        }

        // TODO: enums

        for assert in hir_session.asserts.iter_mut() {
            if let Err(()) = self.resolve_assert(assert) {
                has_error = true;
            }
        }

        for type_assertion in hir_session.type_assertions.iter_mut() {
            if let Err(()) = self.resolve_type(&mut type_assertion.r#type, &mut vec![]) {
                has_error = true;
            }

            else if let Err(()) = self.check_type_annotation_path(&type_assertion.r#type) {
                has_error = true;
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    pub fn resolve_let(&mut self, r#let: &mut Let) -> Result<(), ()> {
        let mut has_error = false;

        if let Some(type_annot) = &mut r#let.type_annot {
            if let Err(()) = self.resolve_type(type_annot, &mut vec![]) {
                has_error = true;
            }

            else if let Err(()) = self.check_type_annotation_path(&type_annot) {
                has_error = true;
            }
        }

        if let Err(()) = self.resolve_expr(&mut r#let.value) {
            has_error = true;
        }

        else if let Err(()) = self.check_expr_path(&r#let.value) {
            has_error = true;
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    pub fn resolve_func(&mut self, func: &mut Func) -> Result<(), ()> {
        let mut has_error = false;

        for param in func.params.iter_mut() {
            if let Some(type_annot) = &mut param.type_annot {
                if let Err(()) = self.resolve_type(type_annot, &mut vec![]) {
                    has_error = true;
                }

                else if let Err(()) = self.check_type_annotation_path(type_annot) {
                    has_error = true;
                }
            }
        }

        if let Some(type_annot) = &mut func.type_annot {
            if let Err(()) = self.resolve_type(type_annot, &mut vec![]) {
                has_error = true;
            }

            else if let Err(()) = self.check_type_annotation_path(type_annot) {
                has_error = true;
            }
        }

        if let Err(()) = self.resolve_expr(&mut func.value) {
            has_error = true;
        }

        else if let Err(()) = self.check_expr_path(&func.value) {
            has_error = true;
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    pub fn resolve_struct(&mut self, r#struct: &mut Struct) -> Result<(), ()> {
        let mut has_error = false;

        for field in r#struct.fields.iter_mut() {
            if let Some(type_annot) = &mut field.type_annot {
                if let Err(()) = self.resolve_type(type_annot, &mut vec![]) {
                    has_error = true;
                }

                else if let Err(()) = self.check_type_annotation_path(type_annot) {
                    has_error = true;
                }
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    pub fn resolve_assert(&mut self, assert: &mut Assert) -> Result<(), ()> {
        let mut has_error = false;

        if let Some(note) = &mut assert.note {
            if let Err(()) = self.resolve_expr(note) {
                has_error = true;
            }

            else if let Err(()) = self.check_expr_path(note) {
                has_error = true;
            }
        }

        if let Err(()) = self.resolve_expr(&mut assert.value) {
            has_error = true;
        }

        else if let Err(()) = self.check_expr_path(&assert.value) {
            has_error = true;
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }
}
