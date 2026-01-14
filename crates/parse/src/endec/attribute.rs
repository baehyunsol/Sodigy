use crate::{
    Attribute,
    Decorator,
    DecoratorArg,
    DocComment,
    DocCommentLine,
    Expr,
    Type,
    Visibility,
};
use sodigy_endec::{DecodeError, Endec};
use sodigy_error::Error;
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Attribute {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.doc_comment.encode_impl(buffer);
        self.decorators.encode_impl(buffer);
        self.visibility.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (doc_comment, cursor) = Option::<DocComment>::decode_impl(buffer, cursor)?;
        let (decorators, cursor) = Vec::<Decorator>::decode_impl(buffer, cursor)?;
        let (visibility, cursor) = Option::<Visibility>::decode_impl(buffer, cursor)?;

        Ok((
            Attribute {
                doc_comment,
                decorators,
                visibility,
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
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (args, cursor) = Option::<Vec<DecoratorArg>>::decode_impl(buffer, cursor)?;
        let (arg_group_span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;

        Ok((
            Decorator {
                name,
                name_span,
                args,
                arg_group_span,
            },
            cursor,
        ))
    }
}

impl Endec for DecoratorArg {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.keyword.encode_impl(buffer);
        self.expr.encode_impl(buffer);
        self.r#type.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (keyword, cursor) = Option::<(InternedString, Span)>::decode_impl(buffer, cursor)?;
        let (expr, cursor) = Result::<Expr, Vec<Error>>::decode_impl(buffer, cursor)?;
        let (r#type, cursor) = Result::<Type, Vec<Error>>::decode_impl(buffer, cursor)?;

        Ok((
            DecoratorArg {
                keyword,
                expr,
                r#type,
            },
            cursor,
        ))
    }
}

impl Endec for Visibility {
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
            Visibility {
                keyword_span,
                args,
                arg_group_span
            },
            cursor,
        ))
    }
}
