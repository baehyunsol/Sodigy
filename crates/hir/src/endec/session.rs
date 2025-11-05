use crate::{
    Alias,
    Assert,
    Enum,
    Func,
    Let,
    Module,
    Session,
    Struct,
    Use,
};
use sodigy_endec::{DecodeError, DumpIr, Endec};
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

        self.lets.encode_impl(buffer);
        self.funcs.encode_impl(buffer);
        self.structs.encode_impl(buffer);
        self.enums.encode_impl(buffer);
        self.asserts.encode_impl(buffer);
        self.aliases.encode_impl(buffer);
        self.uses.encode_impl(buffer);
        self.modules.encode_impl(buffer);
        self.lang_items.encode_impl(buffer);
        self.errors.encode_impl(buffer);
        self.warnings.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (lets, cursor) = Vec::<Let>::decode_impl(buffer, cursor)?;
        let (funcs, cursor) = Vec::<Func>::decode_impl(buffer, cursor)?;
        let (structs, cursor) = Vec::<Struct>::decode_impl(buffer, cursor)?;
        let (enums, cursor) = Vec::<Enum>::decode_impl(buffer, cursor)?;
        let (asserts, cursor) = Vec::<Assert>::decode_impl(buffer, cursor)?;
        let (aliases, cursor) = Vec::<Alias>::decode_impl(buffer, cursor)?;
        let (uses, cursor) = Vec::<Use>::decode_impl(buffer, cursor)?;
        let (modules, cursor) = Vec::<Module>::decode_impl(buffer, cursor)?;
        let (lang_items, cursor) = HashMap::<String, Span>::decode_impl(buffer, cursor)?;
        let (errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
        let (warnings, cursor) = Vec::<Warning>::decode_impl(buffer, cursor)?;

        Ok((
            Session {
                // You have to set this after decoding it.
                intermediate_dir: String::new(),
                name_stack: vec![],
                func_default_values: vec![],
                is_in_debug_context: false,
                lets,
                funcs,
                structs,
                enums,
                asserts,
                aliases,
                uses,
                modules,
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

