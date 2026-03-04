use crate::Session;
use sodigy_endec::{DecodeError, DumpSession, Endec};
use sodigy_error::{Error, Warning};
use sodigy_hir::{EnumShape, FuncShape, Poly, StructShape};
use sodigy_mir::Type;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::{HashMap, HashSet};

impl Endec for Session {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        // The other fields are just tmp values.
        self.types.encode_impl(buffer);
        self.generic_args.encode_impl(buffer);
        self.func_shapes.encode_impl(buffer);
        self.struct_shapes.encode_impl(buffer);
        self.enum_shapes.encode_impl(buffer);
        self.generic_def_span_rev.encode_impl(buffer);
        self.polys.encode_impl(buffer);
        self.span_string_map.encode_impl(buffer);
        self.lang_items.encode_impl(buffer);
        self.errors.encode_impl(buffer);
        self.warnings.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (types, cursor) = HashMap::<Span, Type>::decode_impl(buffer, cursor)?;
        let (generic_args, cursor) = HashMap::<(Span, Span), Type>::decode_impl(buffer, cursor)?;
        let (func_shapes, cursor) = HashMap::<Span, FuncShape>::decode_impl(buffer, cursor)?;
        let (struct_shapes, cursor) = HashMap::<Span, StructShape>::decode_impl(buffer, cursor)?;
        let (enum_shapes, cursor) = HashMap::<Span, EnumShape>::decode_impl(buffer, cursor)?;
        let (generic_def_span_rev, cursor) = HashMap::<Span, Span>::decode_impl(buffer, cursor)?;
        let (polys, cursor) = HashMap::<Span, Poly>::decode_impl(buffer, cursor)?;
        let (span_string_map, cursor) = HashMap::<Span, InternedString>::decode_impl(buffer, cursor)?;
        let (lang_items, cursor) = HashMap::<String, Span>::decode_impl(buffer, cursor)?;
        let (errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
        let (warnings, cursor) = Vec::<Warning>::decode_impl(buffer, cursor)?;

        Ok((
            Session {
                type_vars: HashMap::new(),
                type_var_refs: HashMap::new(),
                maybe_never_type: HashMap::new(),
                blocked_type_vars: HashSet::new(),
                pattern_name_bindings: HashSet::new(),
                solved_generic_args: HashSet::new(),
                types,
                generic_args,
                func_shapes,
                struct_shapes,
                enum_shapes,
                generic_def_span_rev,
                polys,
                span_string_map,
                lang_items,
                intermediate_dir: String::new(),
                type_errors: vec![],
                type_warnings: vec![],
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
            "{{ types: {:?}, generic_args: {:?} }}",
            self.types,
            self.generic_args,
        );
        let mut c = sodigy_prettify::Context::new(s.as_bytes().to_vec());
        c.step_all();
        c.output().to_vec()
    }
}
