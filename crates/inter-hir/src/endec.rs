use crate::Session;
use sodigy_endec::{DecodeError, DumpSession, Endec};
use sodigy_error::{Error, Warning};
use sodigy_hir::{Expr, FuncShape, Poly, StructShape};
use sodigy_name_analysis::NameKind;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

impl Endec for Session {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        // changes everytime
        // self.intermediate_dir.encode_impl(buffer);

        self.func_shapes.encode_impl(buffer);
        self.struct_shapes.encode_impl(buffer);
        self.name_aliases.encode_impl(buffer);
        self.type_aliases.encode_impl(buffer);

        self.item_name_map.encode_impl(buffer);
        self.lang_items.encode_impl(buffer);
        self.polys.encode_impl(buffer);
        self.poly_impls.encode_impl(buffer);
        self.errors.encode_impl(buffer);
        self.warnings.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (func_shapes, cursor) = HashMap::<Span, FuncShape>::decode_impl(buffer, cursor)?;
        let (struct_shapes, cursor) = HashMap::<Span, StructShape>::decode_impl(buffer, cursor)?;
        let (name_aliases, cursor) = HashMap::<_, _>::decode_impl(buffer, cursor)?;
        let (type_aliases, cursor) = HashMap::<_, _>::decode_impl(buffer, cursor)?;
        let (item_name_map, cursor) = HashMap::<Span, (NameKind, HashMap<InternedString, (Span, NameKind)>)>::decode_impl(buffer, cursor)?;
        let (lang_items, cursor) = HashMap::<String, Span>::decode_impl(buffer, cursor)?;
        let (polys, cursor) = HashMap::<Span, Poly>::decode_impl(buffer, cursor)?;
        let (poly_impls, cursor) = Vec::<(Expr, Span)>::decode_impl(buffer, cursor)?;
        let (errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
        let (warnings, cursor) = Vec::<Warning>::decode_impl(buffer, cursor)?;

        Ok((
            Session {
                // You have to set this after decoding it.
                intermediate_dir: String::new(),
                func_shapes,
                struct_shapes,
                name_aliases,
                type_aliases,
                item_name_map,
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
            "{{ func_shapes: {:?}, struct_shapes: {:?}, name_aliases: {:?}, type_aliases: {:?}, item_name_map: {:?}, lang_items: {:?}, polys: {:?} }}",
            self.func_shapes,
            self.struct_shapes,
            self.name_aliases,
            self.type_aliases,
            self.item_name_map,
            self.lang_items,
            self.polys,
        );
        let mut c = sodigy_prettify::Context::new(s.as_bytes().to_vec());
        c.step_all();
        c.output().to_vec()
    }
}
