use crate::{Assert, Func, Let, Session, Type};
use sodigy_endec::{DecodeError, DumpIr, Endec};
use sodigy_error::{Error, Warning};
use sodigy_hir::{FuncArgDef, GenericDef, StructField};
use sodigy_span::Span;
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
        self.asserts.encode_impl(buffer);

        // These 2 are likely to be empty... but encoding/decoding an empty
        // map is very cheap, so who cares!
        self.types.encode_impl(buffer);
        self.generic_instances.encode_impl(buffer);

        // you can re-construct it from scratch
        // self.span_string_map.encode_impl(buffer);

        self.lang_items.encode_impl(buffer);
        self.errors.encode_impl(buffer);
        self.warnings.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (func_shapes, cursor) = HashMap::<Span, (Vec<FuncArgDef<()>>, Vec<GenericDef>)>::decode_impl(buffer, cursor)?;
        let (struct_shapes, cursor) = HashMap::<Span, (Vec<StructField<()>>, Vec<GenericDef>)>::decode_impl(buffer, cursor)?;
        let (generic_def_span_rev, cursor) = HashMap::<Span, Span>::decode_impl(buffer, cursor)?;
        let (lets, cursor) = Vec::<Let>::decode_impl(buffer, cursor)?;
        let (funcs, cursor) = Vec::<Func>::decode_impl(buffer, cursor)?;
        let (asserts, cursor) = Vec::<Assert>::decode_impl(buffer, cursor)?;
        let (types, cursor) = HashMap::<Span, Type>::decode_impl(buffer, cursor)?;
        let (generic_instances, cursor) = HashMap::<(Span, Span), Type>::decode_impl(buffer, cursor)?;
        let (lang_items, cursor) = HashMap::<String, Span>::decode_impl(buffer, cursor)?;
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
                asserts,
                types,
                generic_instances,
                span_string_map: None,
                lang_items,
                errors,
                warnings,
            },
            cursor,
        ))
    }
}

impl DumpIr for Session {
    fn dump_ir(&self) -> Vec<u8> {
        let s = format!(
            "{}lets: {:?}, funcs: {:?}, asserts: {:?}{}",
            "{",
            self.lets,
            self.funcs,
            self.asserts,
            "}",
        );
        let mut c = sodigy_prettify::Context::new(s.as_bytes().to_vec());
        c.step_all();
        c.output().to_vec()
    }
}
