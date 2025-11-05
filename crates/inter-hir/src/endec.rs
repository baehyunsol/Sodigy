use crate::Session;
use sodigy_endec::{DecodeError, DumpIr, Endec};
use sodigy_error::{Error, Warning};
use sodigy_hir::{FuncArgDef, GenericDef, StructField};
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

        // tmp values (per file)
        // self.name_aliases.encode_impl(buffer);
        // self.type_aliases.encode_impl(buffer);

        self.module_name_map.encode_impl(buffer);
        self.lang_items.encode_impl(buffer);
        self.errors.encode_impl(buffer);
        self.warnings.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (func_shapes, cursor) = HashMap::<Span, (Vec<FuncArgDef<()>>, Vec<GenericDef>)>::decode_impl(buffer, cursor)?;
        let (struct_shapes, cursor) = HashMap::<Span, (Vec<StructField<()>>, Vec<GenericDef>)>::decode_impl(buffer, cursor)?;
        let (module_name_map, cursor) = HashMap::<Span, (Span, NameKind, HashMap<InternedString, Span>)>::decode_impl(buffer, cursor)?;
        let (lang_items, cursor) = HashMap::<String, Span>::decode_impl(buffer, cursor)?;
        let (errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
        let (warnings, cursor) = Vec::<Warning>::decode_impl(buffer, cursor)?;

        Ok((
            Session {
                // You have to set this after decoding it.
                intermediate_dir: String::new(),
                func_shapes,
                struct_shapes,
                name_aliases: HashMap::new(),
                type_aliases: HashMap::new(),
                module_name_map,
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
        todo!()
    }
}
