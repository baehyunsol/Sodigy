use crate::{
    Assert,
    Enum,
    Func,
    Let,
    Session,
    Struct,
    Type,
    TypeAssertion,
    dump::{dump_assert, dump_func, dump_let},
};
use sodigy_endec::{DecodeError, DumpSession, Endec, IndentedLines};
use sodigy_error::{Error, Warning};
use sodigy_hir::{FuncShape, Poly, StructShape};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

impl Endec for Session {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        // changes everytime
        // self.intermediate_dir.encode_impl(buffer);

        // TODO: aren't these too expensive to save per-file?
        self.func_shapes.encode_impl(buffer);
        self.struct_shapes.encode_impl(buffer);
        self.generic_def_span_rev.encode_impl(buffer);

        self.lets.encode_impl(buffer);
        self.funcs.encode_impl(buffer);
        self.enums.encode_impl(buffer);
        self.structs.encode_impl(buffer);
        self.asserts.encode_impl(buffer);
        self.aliases.encode_impl(buffer);
        self.type_assertions.encode_impl(buffer);

        // These 2 are likely to be empty... but encoding/decoding an empty
        // map is very cheap, so who cares!
        self.types.encode_impl(buffer);
        self.generic_args.encode_impl(buffer);

        // you can re-construct it from scratch
        // self.span_string_map.encode_impl(buffer);

        self.lang_items.encode_impl(buffer);
        self.polys.encode_impl(buffer);
        self.errors.encode_impl(buffer);
        self.warnings.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (func_shapes, cursor) = HashMap::<Span, FuncShape>::decode_impl(buffer, cursor)?;
        let (struct_shapes, cursor) = HashMap::<Span, StructShape>::decode_impl(buffer, cursor)?;
        let (generic_def_span_rev, cursor) = HashMap::<Span, Span>::decode_impl(buffer, cursor)?;
        let (lets, cursor) = Vec::<Let>::decode_impl(buffer, cursor)?;
        let (funcs, cursor) = Vec::<Func>::decode_impl(buffer, cursor)?;
        let (enums, cursor) = Vec::<Enum>::decode_impl(buffer, cursor)?;
        let (structs, cursor) = Vec::<Struct>::decode_impl(buffer, cursor)?;
        let (asserts, cursor) = Vec::<Assert>::decode_impl(buffer, cursor)?;
        let (aliases, cursor) = Vec::<(InternedString, Span)>::decode_impl(buffer, cursor)?;
        let (type_assertions, cursor) = Vec::<TypeAssertion>::decode_impl(buffer, cursor)?;
        let (types, cursor) = HashMap::<Span, Type>::decode_impl(buffer, cursor)?;
        let (generic_args, cursor) = HashMap::<(Span, Span), Type>::decode_impl(buffer, cursor)?;
        let (lang_items, cursor) = HashMap::<String, Span>::decode_impl(buffer, cursor)?;
        let (polys, cursor) = HashMap::<Span, Poly>::decode_impl(buffer, cursor)?;
        let (errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
        let (warnings, cursor) = Vec::<Warning>::decode_impl(buffer, cursor)?;

        Ok((
            Session {
                // You have to set this after decoding it.
                intermediate_dir: String::new(),
                func_shapes,
                struct_shapes,
                generic_def_span_rev,
                lets,
                funcs,
                enums,
                structs,
                asserts,
                aliases,
                type_assertions,
                types,
                generic_args,
                span_string_map: None,
                lang_items,
                polys,
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
            dump_let(r#let, &mut indented_lines, self, true);
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
