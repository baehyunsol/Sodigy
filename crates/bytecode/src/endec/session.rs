use crate::{Assert, Func, Let, Session};
use sodigy_endec::{DecodeError, DumpSession, Endec};
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

        self.funcs.encode_impl(buffer);
        self.asserts.encode_impl(buffer);
        self.lets.encode_impl(buffer);

        // tmp data
        // self.local_values.encode_impl(buffer);
        // self.stack_offset.encode_impl(buffer);

        self.intrinsics.encode_impl(buffer);
        self.errors.encode_impl(buffer);
        self.warnings.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (funcs, cursor) = Vec::<Func>::decode_impl(buffer, cursor)?;
        let (asserts, cursor) = Vec::<Assert>::decode_impl(buffer, cursor)?;
        let (lets, cursor) = Vec::<Let>::decode_impl(buffer, cursor)?;
        let (intrinsics, cursor) = HashMap::<Span, Intrinsic>::decode_impl(buffer, cursor)?;
        let (errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
        let (warnings, cursor) = Vec::<Warning>::decode_impl(buffer, cursor)?;

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
                stack_offset: 0,

                intrinsics,
                errors,
                warnings,

                // worker will load this
                global_context: GlobalContext::new(),
            },
            cursor,
        ))
    }
}

impl DumpSession for Session<'_, '_> {
    fn dump_session(&self) -> Vec<u8> {
        let mut lines = vec![];

        for func in self.funcs.iter() {
            lines.push(format!("// name: {}", func.name.unintern_or_default(&self.intermediate_dir)));
            lines.push(format!("// name_span: {:?}", func.name_span));
            lines.push(format!("func {:09x}:", func.name_span.hash() & 0xfff_fff_fff));

            for bytecode in func.bytecodes.iter() {
                lines.push(format!("    {bytecode}"));
            }

            lines.push(String::new());
        }

        for r#let in self.lets.iter() {
            lines.push(format!("// name: {}", r#let.name.unintern_or_default(&self.intermediate_dir)));
            lines.push(format!("// name_span: {:?}", r#let.name_span));
            lines.push(format!("data {:09x}:", r#let.name_span.hash() & 0xfff_fff_fff));

            for bytecode in r#let.bytecodes.iter() {
                lines.push(format!("    {bytecode}"));
            }

            lines.push(String::new());
        }

        for assert in self.asserts.iter() {
            lines.push(format!("// name: {}", assert.name.unintern_or_default(&self.intermediate_dir)));
            lines.push(format!("// keyword_span: {:?}", assert.keyword_span));
            lines.push(format!("assert {:09x}:", assert.keyword_span.hash() & 0xfff_fff_fff));

            for bytecode in assert.bytecodes.iter() {
                lines.push(format!("    {bytecode}"));
            }

            lines.push(String::new());
        }

        let s = format!(
            "{{ lets: {:?}, funcs: {:?}, asserts: {:?} }}",
            self.lets,
            self.funcs,
            self.asserts,
        );
        let mut c = sodigy_prettify::Context::new(s.as_bytes().to_vec());
        c.step_all();

        vec![
            lines.join("\n").into_bytes(),
            c.output().to_vec(),
        ].concat()
    }
}
