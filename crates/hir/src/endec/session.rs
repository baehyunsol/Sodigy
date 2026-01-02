use crate::{
    Alias,
    Assert,
    Enum,
    Expr,
    Func,
    Let,
    Module,
    Poly,
    Session,
    Struct,
    Use,
    dump::{dump_assert, dump_func, dump_let},
};
use sodigy_endec::{DecodeError, DumpSession, Endec, IndentedLines};
use sodigy_error::{Error, Warning};
use sodigy_span::Span;
use std::collections::HashMap;

impl Endec for Session {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        // changes everytime
        // self.intermediate_dir.encode_impl(buffer);

        // must be empty when encoding
        // self.name_stack.encode_impl(buffer);
        // self.func_default_values.encode_impl(buffer);
        // self.is_in_debug_context.encode_impl(buffer);

        // doesn't have to be stored on disk
        // self.attribute_rule_cache.encode_impl(buffer);

        self.is_std.encode_impl(buffer);
        self.lets.encode_impl(buffer);
        self.funcs.encode_impl(buffer);
        self.structs.encode_impl(buffer);
        self.enums.encode_impl(buffer);
        self.asserts.encode_impl(buffer);
        self.aliases.encode_impl(buffer);
        self.uses.encode_impl(buffer);
        self.modules.encode_impl(buffer);
        self.lang_items.encode_impl(buffer);
        self.polys.encode_impl(buffer);
        self.poly_impls.encode_impl(buffer);
        self.errors.encode_impl(buffer);
        self.warnings.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (is_std, cursor) = bool::decode_impl(buffer, cursor)?;
        let (lets, cursor) = Vec::<Let>::decode_impl(buffer, cursor)?;
        let (funcs, cursor) = Vec::<Func>::decode_impl(buffer, cursor)?;
        let (structs, cursor) = Vec::<Struct>::decode_impl(buffer, cursor)?;
        let (enums, cursor) = Vec::<Enum>::decode_impl(buffer, cursor)?;
        let (asserts, cursor) = Vec::<Assert>::decode_impl(buffer, cursor)?;
        let (aliases, cursor) = Vec::<Alias>::decode_impl(buffer, cursor)?;
        let (uses, cursor) = Vec::<Use>::decode_impl(buffer, cursor)?;
        let (modules, cursor) = Vec::<Module>::decode_impl(buffer, cursor)?;
        let (lang_items, cursor) = HashMap::<String, Span>::decode_impl(buffer, cursor)?;
        let (polys, cursor) = HashMap::<Span, Poly>::decode_impl(buffer, cursor)?;
        let (poly_impls, cursor) = Vec::<(Expr, Span)>::decode_impl(buffer, cursor)?;
        let (errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
        let (warnings, cursor) = Vec::<Warning>::decode_impl(buffer, cursor)?;

        Ok((
            Session {
                // You have to set this after decoding it.
                intermediate_dir: String::new(),
                name_stack: vec![],
                func_default_values: vec![],
                attribute_rule_cache: HashMap::new(),
                is_in_debug_context: false,
                nested_pipeline_depth: 0,
                is_std,
                lets,
                funcs,
                structs,
                enums,
                asserts,
                aliases,
                uses,
                modules,
                lang_items,
                polys,
                poly_impls,
                errors,
                warnings,
            },
            cursor,
        ))
    }
}

impl DumpSession for Session {
    fn dump_session(&self) -> Vec<u8> {
        let s = format!(
            "{{ lets: {:?}, funcs: {:?}, asserts: {:?} }}",
            self.lets,
            self.funcs,
            self.asserts,
        );
        let mut c = sodigy_prettify::Context::new(s.as_bytes().to_vec());
        c.step_all();
        let s = String::from_utf8(c.output().to_vec()).unwrap();
        let mut indented_lines = IndentedLines::new();

        for r#let in self.lets.iter() {
            dump_let(r#let, &mut indented_lines, self);
        }

        for func in self.funcs.iter() {
            dump_func(func, &mut indented_lines, self);
        }

        for assert in self.asserts.iter() {
            dump_assert(assert, &mut indented_lines, self);
        }

        format!("{}\n\nlet session = {s};", indented_lines.dump()).into_bytes()
    }
}
