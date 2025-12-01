use crate::{Assert, Func, Let, Session};
use sodigy_endec::{DecodeError, DumpIr, Endec};
use sodigy_mir::Intrinsic;
use sodigy_span::Span;
use std::collections::HashMap;

impl Endec for Session {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        // changes everytime
        // self.intermediate_dir.encode_impl(buffer);

        // tmp data
        // self.label_counter.encode_impl(buffer);

        self.funcs.encode_impl(buffer);
        self.asserts.encode_impl(buffer);
        self.lets.encode_impl(buffer);

        // tmp data
        // self.local_values.encode_impl(buffer);
        // self.drop_types.encode_impl(buffer);

        self.intrinsics.encode_impl(buffer);
        self.lang_items.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (funcs, cursor) = Vec::<Func>::decode_impl(buffer, cursor)?;
        let (asserts, cursor) = Vec::<Assert>::decode_impl(buffer, cursor)?;
        let (lets, cursor) = Vec::<Let>::decode_impl(buffer, cursor)?;
        let (intrinsics, cursor) = HashMap::<Span, Intrinsic>::decode_impl(buffer, cursor)?;
        let (lang_items, cursor) = HashMap::<String, Span>::decode_impl(buffer, cursor)?;

        Ok((
            Session {
                // You have to set this after decoding it.
                intermediate_dir: String::new(),

                // tmp data
                label_counter: 0,

                funcs,
                asserts,
                lets,

                // tmp data
                local_values: HashMap::new(),
                drop_types: HashMap::new(),

                intrinsics,
                lang_items,
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
