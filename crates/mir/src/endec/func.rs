use crate::{Expr, Func};
use sodigy_endec::{DecodeError, Endec};
use sodigy_hir::{FuncArgDef, GenericDef};
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Func {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.generics.encode_impl(buffer);
        self.args.encode_impl(buffer);
        self.type_annotation_span.encode_impl(buffer);
        self.value.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (generics, cursor) = Vec::<GenericDef>::decode_impl(buffer, cursor)?;
        let (args, cursor) = Vec::<FuncArgDef<()>>::decode_impl(buffer, cursor)?;
        let (type_annotation_span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;
        let (value, cursor) = Expr::decode_impl(buffer, cursor)?;

        Ok((
            Func {
                name,
                name_span,
                generics,
                args,
                type_annotation_span,
                value,
            },
            cursor,
        ))
    }
}
