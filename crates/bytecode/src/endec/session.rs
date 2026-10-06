use crate::{Assert, Func, Let, Session};
use sodigy_endec::{DecodeError, Endec};
use sodigy_error::{Error, Warning};
use sodigy_mir::{GlobalContext, Intrinsic};
use sodigy_span::Span;
use std::collections::HashMap;

impl Endec for Session<'_, '_> {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        // changes everytime
        // self.intermediate_dir.encode_impl(buffer);

        // tmp data
        // self.label_counter.encode_impl(buffer);
        // self.ssa_counter.encode_impl(buffer);
        // self.ssa_map.encode_impl(buffer);
        // self.number_to_expr_hash.encode_impl(buffer);
        // self.string_to_expr_hash.encode_impl(buffer);
        // self.data_section.encode_impl(buffer);

        self.lets.encode_impl(buffer);
        self.funcs.encode_impl(buffer);
        self.asserts.encode_impl(buffer);
        self.intrinsics.encode_impl(buffer);
        self.errors.encode_impl(buffer);
        self.warnings.encode_impl(buffer);
        self.debug_info.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (lets, cursor) = Vec::<Let>::decode_impl(buffer, cursor)?;
        let (funcs, cursor) = Vec::<Func>::decode_impl(buffer, cursor)?;
        let (asserts, cursor) = Vec::<Assert>::decode_impl(buffer, cursor)?;
        let (intrinsics, cursor) = HashMap::<Span, Intrinsic>::decode_impl(buffer, cursor)?;
        let (errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
        let (warnings, cursor) = Vec::<Warning>::decode_impl(buffer, cursor)?;
        let (debug_info, cursor) = bool::decode_impl(buffer, cursor)?;

        Ok((
            Session {
                // You have to set this after decoding it.
                intermediate_dir: String::new(),

                // tmp data
                label_counter: 0,
                ssa_counter: 0,
                ssa_map: HashMap::new(),
                number_to_expr_hash: HashMap::new(),
                string_to_expr_hash: HashMap::new(),
                data_section: HashMap::new(),

                lets,
                funcs,
                asserts,
                intrinsics,
                errors,
                warnings,

                // worker will load this
                global_context: GlobalContext::new(),
                debug_info,
            },
            cursor,
        ))
    }
}
