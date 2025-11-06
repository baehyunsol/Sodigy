use crate::{Error, ErrorKind, ErrorToken};
use sodigy_endec::{DecodeError, Endec};
use sodigy_file::ModulePath;
use sodigy_name_analysis::NameKind;
use sodigy_span::RenderableSpan;
use sodigy_string::InternedString;
use sodigy_token::{Delim, InfixOp, Keyword, Punct};

impl Endec for Error {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.kind.encode_impl(buffer);
        self.spans.encode_impl(buffer);
        self.note.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (kind, cursor) = ErrorKind::decode_impl(buffer, cursor)?;
        let (spans, cursor) = Vec::<RenderableSpan>::decode_impl(buffer, cursor)?;
        let (note, cursor) = Option::<String>::decode_impl(buffer, cursor)?;
        Ok((Error { kind, spans, note }, cursor))
    }
}

impl Endec for ErrorKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            ErrorKind::InvalidNumberLiteral => {
                buffer.push(0);
            },
            ErrorKind::InvalidStringLiteralPrefix => {
                buffer.push(1);
            },
            ErrorKind::InvalidCharacterInIdentifier(ch) => {
                buffer.push(2);
                ch.encode_impl(buffer);
            },
            ErrorKind::WrongNumberOfQuotesInRawStringLiteral => {
                buffer.push(3);
            },
            ErrorKind::UnterminatedStringLiteral => {
                buffer.push(4);
            },
            ErrorKind::InvalidCharLiteral => {
                buffer.push(5);
            },
            ErrorKind::InvalidCharLiteralPrefix => {
                buffer.push(6);
            },
            ErrorKind::UnterminatedCharLiteral => {
                buffer.push(7);
            },
            ErrorKind::InvalidByteLiteral => {
                buffer.push(8);
            },
            ErrorKind::InvalidEscape => {
                buffer.push(9);
            },
            ErrorKind::EmptyCharLiteral => {
                buffer.push(10);
            },
            ErrorKind::UnterminatedBlockComment => {
                buffer.push(11);
            },
            ErrorKind::InvalidUtf8 => {
                buffer.push(12);
            },
            ErrorKind::InvalidUnicodeCharacter => {
                buffer.push(13);
            },
            ErrorKind::InvalidUnicodeEscape => {
                buffer.push(14);
            },
            ErrorKind::UnmatchedGroup { expected, got } => {
                buffer.push(15);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::TooManyQuotes => {
                buffer.push(16);
            },
            ErrorKind::UnclosedDelimiter(delim) => {
                buffer.push(17);
                delim.encode_impl(buffer);
            },
            ErrorKind::UnexpectedToken { expected, got } => {
                buffer.push(18);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::UnexpectedEof { expected } => {
                buffer.push(19);
                expected.encode_impl(buffer);
            },
            ErrorKind::UnexpectedEog { expected } => {
                buffer.push(20);
                expected.encode_impl(buffer);
            },
            ErrorKind::DocCommentForNothing => {
                buffer.push(21);
            },
            ErrorKind::DocCommentNotAllowed => {
                buffer.push(22);
            },
            ErrorKind::DecoratorNotAllowed => {
                buffer.push(23);
            },
            ErrorKind::CannotBePublic => {
                buffer.push(24);
            },
            ErrorKind::BlockWithoutValue => {
                buffer.push(25);
            },
            ErrorKind::StructWithoutField => {
                buffer.push(26);
            },
            ErrorKind::EmptyCurlyBraceBlock => {
                buffer.push(27);
            },
            ErrorKind::PositionalArgAfterKeywordArg => {
                buffer.push(28);
            },
            ErrorKind::NonDefaultValueAfterDefaultValue => {
                buffer.push(29);
            },
            ErrorKind::CannotDeclareInlineModule => {
                buffer.push(30);
            },
            ErrorKind::InclusiveRangeWithNoEnd => {
                buffer.push(31);
            },
            ErrorKind::AstPatternTypeError => {
                buffer.push(32);
            },
            ErrorKind::DifferentNameBindingsInOrPattern => {
                buffer.push(33);
            },
            ErrorKind::InvalidFnType => {
                buffer.push(34);
            },
            ErrorKind::EmptyMatchStatement => {
                buffer.push(35);
            },
            ErrorKind::RedundantDecorator(s) => {
                buffer.push(36);
                s.encode_impl(buffer);
            },
            ErrorKind::InvalidDecorator(s) => {
                buffer.push(37);
                s.encode_impl(buffer);
            },
            ErrorKind::CannotBindNameToAnotherName(s) => {
                buffer.push(38);
                s.encode_impl(buffer);
            },
            ErrorKind::CannotAnnotateType => {
                buffer.push(39);
            },
            ErrorKind::RedundantNameBinding(a, b) => {
                buffer.push(40);
                a.encode_impl(buffer);
                b.encode_impl(buffer);
            },
            ErrorKind::NameCollision { name } => {
                buffer.push(41);
                name.encode_impl(buffer);
            },
            ErrorKind::CyclicLet { names } => {
                buffer.push(42);
                names.encode_impl(buffer);
            },
            ErrorKind::CyclicAlias { names } => {
                buffer.push(43);
                names.encode_impl(buffer);
            },
            ErrorKind::UndefinedName(s) => {
                buffer.push(44);
                s.encode_impl(buffer);
            },
            ErrorKind::KeywordArgumentRepeated(s) => {
                buffer.push(45);
                s.encode_impl(buffer);
            },
            ErrorKind::KeywordArgumentNotAllowed => {
                buffer.push(46);
            },
            ErrorKind::AliasResolveRecursionLimitReached => {
                buffer.push(47);
            },
            ErrorKind::MissingTypeArgument { expected, got } => {
                buffer.push(48);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::UnexpectedTypeArgument { expected, got } => {
                buffer.push(49);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::InvalidKeywordArgument(s) => {
                buffer.push(50);
                s.encode_impl(buffer);
            },
            ErrorKind::MissingArgument { expected, got } => {
                buffer.push(51);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::UnexpectedArgument { expected, got } => {
                buffer.push(52);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::StructFieldRepeated(s) => {
                buffer.push(53);
                s.encode_impl(buffer);
            },
            ErrorKind::MissingStructField(s) => {
                buffer.push(54);
                s.encode_impl(buffer);
            },
            ErrorKind::InvalidStructField(s) => {
                buffer.push(55);
                s.encode_impl(buffer);
            },
            ErrorKind::DependentTypeNotAllowed => {
                buffer.push(56);
            },
            ErrorKind::UnexpectedType { expected, got } => {
                buffer.push(57);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::CannotInferType { id } => {
                buffer.push(58);
                id.encode_impl(buffer);
            },
            ErrorKind::PartiallyInferedType { id, r#type } => {
                buffer.push(59);
                id.encode_impl(buffer);
                r#type.encode_impl(buffer);
            },
            ErrorKind::CannotInferGenericType { id } => {
                buffer.push(60);
                id.encode_impl(buffer);
            },
            ErrorKind::PartiallyInferedGenericType { id, r#type } => {
                buffer.push(61);
                id.encode_impl(buffer);
                r#type.encode_impl(buffer);
            },
            ErrorKind::CannotApplyInfixOp { op, arg_types } => {
                buffer.push(62);
                op.encode_impl(buffer);
                arg_types.encode_impl(buffer);
            },
            ErrorKind::MultipleModuleFiles { module, found_files } => {
                buffer.push(63);
                module.encode_impl(buffer);
                found_files.encode_impl(buffer);
            },
            ErrorKind::ModuleFileNotFound { module, candidates } => {
                buffer.push(64);
                module.encode_impl(buffer);
                candidates.encode_impl(buffer);
            },
            ErrorKind::LibFileNotFound => {
                buffer.push(65);
            },
            ErrorKind::UnusedName { name, kind } => {
                buffer.push(66);
                name.encode_impl(buffer);
                kind.encode_impl(buffer);
            },
            ErrorKind::Todo { message } => {
                buffer.push(67);
                message.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((ErrorKind::InvalidNumberLiteral, cursor + 1)),
            Some(1) => Ok((ErrorKind::InvalidStringLiteralPrefix, cursor + 1)),
            Some(2) => {
                let (ch, cursor) = char::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::InvalidCharacterInIdentifier(ch), cursor))
            },
            Some(3) => Ok((ErrorKind::WrongNumberOfQuotesInRawStringLiteral, cursor + 1)),
            Some(4) => Ok((ErrorKind::UnterminatedStringLiteral, cursor + 1)),
            Some(5) => Ok((ErrorKind::InvalidCharLiteral, cursor + 1)),
            Some(6) => Ok((ErrorKind::InvalidCharLiteralPrefix, cursor + 1)),
            Some(7) => Ok((ErrorKind::UnterminatedCharLiteral, cursor + 1)),
            Some(8) => Ok((ErrorKind::InvalidByteLiteral, cursor + 1)),
            Some(9) => Ok((ErrorKind::InvalidEscape, cursor + 1)),
            Some(10) => Ok((ErrorKind::EmptyCharLiteral, cursor + 1)),
            Some(11) => Ok((ErrorKind::UnterminatedBlockComment, cursor + 1)),
            Some(12) => Ok((ErrorKind::InvalidUtf8, cursor + 1)),
            Some(13) => Ok((ErrorKind::InvalidUnicodeCharacter, cursor + 1)),
            Some(14) => Ok((ErrorKind::InvalidUnicodeEscape, cursor + 1)),
            Some(15) => {
                let (expected, cursor) = u8::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = u8::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnmatchedGroup { expected, got }, cursor))
            },
            Some(16) => Ok((ErrorKind::TooManyQuotes, cursor + 1)),
            Some(17) => {
                let (delim, cursor) = u8::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::UnclosedDelimiter(delim), cursor))
            },
            Some(18) => {
                let (expected, cursor) = ErrorToken::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = ErrorToken::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnexpectedToken { expected, got }, cursor))
            },
            Some(19) => {
                let (expected, cursor) = ErrorToken::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::UnexpectedEof { expected }, cursor))
            },
            Some(20) => {
                let (expected, cursor) = ErrorToken::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::UnexpectedEog { expected }, cursor))
            },
            Some(21) => Ok((ErrorKind::DocCommentForNothing, cursor + 1)),
            Some(22) => Ok((ErrorKind::DocCommentNotAllowed, cursor + 1)),
            Some(23) => Ok((ErrorKind::DecoratorNotAllowed, cursor + 1)),
            Some(24) => Ok((ErrorKind::CannotBePublic, cursor + 1)),
            Some(25) => Ok((ErrorKind::BlockWithoutValue, cursor + 1)),
            Some(26) => Ok((ErrorKind::StructWithoutField, cursor + 1)),
            Some(27) => Ok((ErrorKind::EmptyCurlyBraceBlock, cursor + 1)),
            Some(28) => Ok((ErrorKind::PositionalArgAfterKeywordArg, cursor + 1)),
            Some(29) => Ok((ErrorKind::NonDefaultValueAfterDefaultValue, cursor + 1)),
            Some(30) => Ok((ErrorKind::CannotDeclareInlineModule, cursor + 1)),
            Some(31) => Ok((ErrorKind::InclusiveRangeWithNoEnd, cursor + 1)),
            Some(32) => Ok((ErrorKind::AstPatternTypeError, cursor + 1)),
            Some(33) => Ok((ErrorKind::DifferentNameBindingsInOrPattern, cursor + 1)),
            Some(34) => Ok((ErrorKind::InvalidFnType, cursor + 1)),
            Some(35) => Ok((ErrorKind::EmptyMatchStatement, cursor + 1)),
            Some(36) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::RedundantDecorator(s), cursor))
            },
            Some(37) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::InvalidDecorator(s), cursor))
            },
            Some(38) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CannotBindNameToAnotherName(s), cursor))
            },
            Some(39) => Ok((ErrorKind::CannotAnnotateType, cursor + 1)),
            Some(40) => {
                let (a, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                let (b, cursor) = InternedString::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::RedundantNameBinding(a, b), cursor))
            },
            Some(41) => {
                let (name, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::NameCollision { name }, cursor))
            },
            Some(42) => {
                let (names, cursor) = Vec::<InternedString>::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CyclicLet { names }, cursor))
            },
            Some(43) => {
                let (names, cursor) = Vec::<InternedString>::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CyclicAlias { names }, cursor))
            },
            Some(44) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::UndefinedName(s), cursor))
            },
            Some(45) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::KeywordArgumentRepeated(s), cursor))
            },
            Some(46) => Ok((ErrorKind::KeywordArgumentNotAllowed, cursor + 1)),
            Some(47) => Ok((ErrorKind::AliasResolveRecursionLimitReached, cursor + 1)),
            Some(48) => {
                let (expected, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::MissingTypeArgument { expected, got }, cursor))
            },
            Some(49) => {
                let (expected, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnexpectedTypeArgument { expected, got }, cursor))
            },
            Some(50) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::InvalidKeywordArgument(s), cursor))
            },
            Some(51) => {
                let (expected, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::MissingArgument { expected, got }, cursor))
            },
            Some(52) => {
                let (expected, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnexpectedArgument { expected, got }, cursor))
            },
            Some(53) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::StructFieldRepeated(s), cursor))
            },
            Some(54) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::MissingStructField(s), cursor))
            },
            Some(55) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::InvalidStructField(s), cursor))
            },
            Some(56) => Ok((ErrorKind::DependentTypeNotAllowed, cursor + 1)),
            Some(57) => {
                let (expected, cursor) = String::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = String::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnexpectedType { expected, got }, cursor))
            },
            Some(58) => {
                let (id, cursor) = Option::<InternedString>::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CannotInferType { id }, cursor))
            },
            Some(59) => {
                let (id, cursor) = Option::<InternedString>::decode_impl(buffer, cursor + 1)?;
                let (r#type, cursor) = String::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::PartiallyInferedType { id, r#type }, cursor))
            },
            Some(60) => {
                let (id, cursor) = Option::<String>::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CannotInferGenericType { id }, cursor))
            },
            Some(61) => {
                let (id, cursor) = Option::<String>::decode_impl(buffer, cursor + 1)?;
                let (r#type, cursor) = String::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::PartiallyInferedGenericType { id, r#type }, cursor))
            },
            Some(62) => {
                let (op, cursor) = InfixOp::decode_impl(buffer, cursor + 1)?;
                let (arg_types, cursor) = Vec::<String>::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::CannotApplyInfixOp { op, arg_types }, cursor))
            },
            Some(63) => {
                let (module, cursor) = ModulePath::decode_impl(buffer, cursor + 1)?;
                let (found_files, cursor) = Vec::<String>::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::MultipleModuleFiles { module, found_files }, cursor))
            },
            Some(64) => {
                let (module, cursor) = ModulePath::decode_impl(buffer, cursor + 1)?;
                let (candidates, cursor) = Vec::<String>::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::ModuleFileNotFound { module, candidates }, cursor))
            },
            Some(65) => Ok((ErrorKind::LibFileNotFound, cursor + 1)),
            Some(66) => {
                let (name, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                let (kind, cursor) = NameKind::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnusedName { name, kind }, cursor))
            },
            Some(67) => {
                let (message, cursor) = String::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::Todo { message }, cursor))
            },
            Some(n @ 68..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for ErrorToken {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            ErrorToken::Nothing => {
                buffer.push(0);
            },
            ErrorToken::Any => {
                buffer.push(1);
            },
            ErrorToken::Character(ch) => {
                buffer.push(2);
                ch.encode_impl(buffer);
            },
            ErrorToken::AnyCharacter => {
                buffer.push(3);
            },
            ErrorToken::Keyword(keyword) => {
                buffer.push(4);
                keyword.encode_impl(buffer);
            },
            ErrorToken::Punct(punct) => {
                buffer.push(5);
                punct.encode_impl(buffer);
            },
            ErrorToken::Group(delim) => {
                buffer.push(6);
                delim.encode_impl(buffer);
            },
            ErrorToken::Identifier => {
                buffer.push(7);
            },
            ErrorToken::Number => {
                buffer.push(8);
            },
            ErrorToken::String => {
                buffer.push(9);
            },
            ErrorToken::TypeAnnotation => {
                buffer.push(10);
            },
            ErrorToken::Declaration => {
                buffer.push(11);
            },
            ErrorToken::Expr => {
                buffer.push(12);
            },
            ErrorToken::Block => {
                buffer.push(13);
            },
            ErrorToken::AssignOrLt => {
                buffer.push(14);
            },
            ErrorToken::BraceOrParenthesis => {
                buffer.push(15);
            },
            ErrorToken::ColonOrComma => {
                buffer.push(16);
            },
            ErrorToken::CommaOrDot => {
                buffer.push(17);
            },
            ErrorToken::CommaOrGt => {
                buffer.push(18);
            },
            ErrorToken::DotOrSemicolon => {
                buffer.push(19);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((ErrorToken::Nothing, cursor + 1)),
            Some(1) => Ok((ErrorToken::Any, cursor + 1)),
            Some(2) => {
                let (ch, cursor) = u8::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorToken::Character(ch), cursor))
            },
            Some(3) => Ok((ErrorToken::AnyCharacter, cursor + 1)),
            Some(4) => {
                let (keyword, cursor) = Keyword::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorToken::Keyword(keyword), cursor))
            },
            Some(5) => {
                let (punct, cursor) = Punct::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorToken::Punct(punct), cursor))
            },
            Some(6) => {
                let (delim, cursor) = Delim::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorToken::Group(delim), cursor))
            },
            Some(7) => Ok((ErrorToken::Identifier, cursor + 1)),
            Some(8) => Ok((ErrorToken::Number, cursor + 1)),
            Some(9) => Ok((ErrorToken::String, cursor + 1)),
            Some(10) => Ok((ErrorToken::TypeAnnotation, cursor + 1)),
            Some(11) => Ok((ErrorToken::Declaration, cursor + 1)),
            Some(12) => Ok((ErrorToken::Expr, cursor + 1)),
            Some(13) => Ok((ErrorToken::Block, cursor + 1)),
            Some(14) => Ok((ErrorToken::AssignOrLt, cursor + 1)),
            Some(15) => Ok((ErrorToken::BraceOrParenthesis, cursor + 1)),
            Some(16) => Ok((ErrorToken::ColonOrComma, cursor + 1)),
            Some(17) => Ok((ErrorToken::CommaOrDot, cursor + 1)),
            Some(18) => Ok((ErrorToken::CommaOrGt, cursor + 1)),
            Some(19) => Ok((ErrorToken::DotOrSemicolon, cursor + 1)),
            Some(n @ 20..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
