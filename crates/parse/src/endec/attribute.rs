use crate::{Attribute, CallArg, Decorator, DocComment, DocCommentLine, Public};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Attribute {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.doc_comment.encode_impl(buffer);
        self.decorators.encode_impl(buffer);
        self.public.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (doc_comment, cursor) = Option::<DocComment>::decode_impl(buffer, cursor)?;
        let (decorators, cursor) = Vec::<Decorator>::decode_impl(buffer, cursor)?;
        let (public, cursor) = Option::<Public>::decode_impl(buffer, cursor)?;

        Ok((
            Attribute {
                doc_comment,
                decorators,
                public,
            },
            cursor,
        ))
    }
}

impl Endec for DocComment {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.0.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (lines, cursor) = Vec::<DocCommentLine>::decode_impl(buffer, cursor)?;

        Ok((
            DocComment(lines),
            cursor,
        ))
    }
}

impl Endec for DocCommentLine {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.content.encode_impl(buffer);
        self.content_span.encode_impl(buffer);
        self.marker_span.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (content, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (content_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (marker_span, cursor) = Span::decode_impl(buffer, cursor)?;

        Ok((
            DocCommentLine {
                content,
                content_span,
                marker_span,
            },
            cursor,
        ))
    }
}

impl Endec for Decorator {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.args.encode_impl(buffer);
        self.arg_group_span.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (name, cursor) = Vec::<(InternedString, Span)>::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (args, cursor) = Option::<Vec<CallArg>>::decode_impl(buffer, cursor)?;
        let (arg_group_span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;

        Ok((
            Decorator {
                name,
                name_span,
                args,
                arg_group_span
            },
            cursor,
        ))
    }
}

impl Endec for Public {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.keyword_span.encode_impl(buffer);
        self.args.encode_impl(buffer);
        self.arg_group_span.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (keyword_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (args, cursor) = Option::<Vec<(InternedString, Span)>>::decode_impl(buffer, cursor)?;
        let (arg_group_span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;

        Ok((
            Public {
                keyword_span,
                args,
                arg_group_span
            },
            cursor,
        ))
    }
}
