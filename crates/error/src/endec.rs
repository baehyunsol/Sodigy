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
            ErrorKind::MissingDocComment => {
                buffer.push(21);
            },
            ErrorKind::DocCommentNotAllowed => {
                buffer.push(22);
            },
            ErrorKind::ModuleDocCommentNotAtTop => {
                buffer.push(23);
            },
            ErrorKind::MissingDecorator(d) => {
                buffer.push(24);
                d.encode_impl(buffer);
            },
            ErrorKind::DecoratorNotAllowed => {
                buffer.push(25);
            },
            ErrorKind::UnexpectedDecorator(d) => {
                buffer.push(26);
                d.encode_impl(buffer);
            },
            ErrorKind::ModuleDecoratorNotAtTop => {
                buffer.push(27);
            },
            ErrorKind::MissingVisibility => {
                buffer.push(28);
            },
            ErrorKind::CannotBePublic => {
                buffer.push(29);
            },
            ErrorKind::BlockWithoutValue => {
                buffer.push(30);
            },
            ErrorKind::StructWithoutField => {
                buffer.push(31);
            },
            ErrorKind::EmptyCurlyBraceBlock => {
                buffer.push(32);
            },
            ErrorKind::PositionalArgAfterKeywordArg => {
                buffer.push(33);
            },
            ErrorKind::NonDefaultValueAfterDefaultValue => {
                buffer.push(34);
            },
            ErrorKind::CannotDeclareInlineModule => {
                buffer.push(35);
            },
            ErrorKind::InclusiveRangeWithNoEnd => {
                buffer.push(36);
            },
            ErrorKind::AstPatternTypeError => {
                buffer.push(37);
            },
            ErrorKind::DifferentNameBindingsInOrPattern => {
                buffer.push(38);
            },
            ErrorKind::InvalidFnType => {
                buffer.push(39);
            },
            ErrorKind::EmptyMatchStatement => {
                buffer.push(40);
            },
            ErrorKind::RedundantDecorator(s) => {
                buffer.push(41);
                s.encode_impl(buffer);
            },
            ErrorKind::InvalidDecorator(s) => {
                buffer.push(42);
                s.encode_impl(buffer);
            },
            ErrorKind::CannotBindNameToAnotherName(s) => {
                buffer.push(43);
                s.encode_impl(buffer);
            },
            ErrorKind::CannotAnnotateType => {
                buffer.push(44);
            },
            ErrorKind::RedundantNameBinding(a, b) => {
                buffer.push(45);
                a.encode_impl(buffer);
                b.encode_impl(buffer);
            },
            ErrorKind::NameCollision { name } => {
                buffer.push(46);
                name.encode_impl(buffer);
            },
            ErrorKind::CyclicLet { names } => {
                buffer.push(47);
                names.encode_impl(buffer);
            },
            ErrorKind::CyclicAlias { names } => {
                buffer.push(48);
                names.encode_impl(buffer);
            },
            ErrorKind::UndefinedName(s) => {
                buffer.push(49);
                s.encode_impl(buffer);
            },
            ErrorKind::KeywordArgumentRepeated(s) => {
                buffer.push(50);
                s.encode_impl(buffer);
            },
            ErrorKind::KeywordArgumentNotAllowed => {
                buffer.push(51);
            },
            ErrorKind::AliasResolveRecursionLimitReached => {
                buffer.push(52);
            },
            ErrorKind::MissingTypeArgument { expected, got } => {
                buffer.push(53);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::UnexpectedTypeArgument { expected, got } => {
                buffer.push(54);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::MissingKeywordArgument(s) => {
                buffer.push(55);
                s.encode_impl(buffer);
            },
            ErrorKind::InvalidKeywordArgument(s) => {
                buffer.push(56);
                s.encode_impl(buffer);
            },
            ErrorKind::MissingArgument { expected, got } => {
                buffer.push(57);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::UnexpectedArgument { expected, got } => {
                buffer.push(58);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::StructFieldRepeated(s) => {
                buffer.push(59);
                s.encode_impl(buffer);
            },
            ErrorKind::MissingStructField(s) => {
                buffer.push(60);
                s.encode_impl(buffer);
            },
            ErrorKind::InvalidStructField(s) => {
                buffer.push(61);
                s.encode_impl(buffer);
            },
            ErrorKind::DependentTypeNotAllowed => {
                buffer.push(62);
            },
            ErrorKind::UnexpectedType { expected, got } => {
                buffer.push(63);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::CannotInferType { id } => {
                buffer.push(64);
                id.encode_impl(buffer);
            },
            ErrorKind::PartiallyInferedType { id, r#type } => {
                buffer.push(65);
                id.encode_impl(buffer);
                r#type.encode_impl(buffer);
            },
            ErrorKind::CannotInferGenericType { id } => {
                buffer.push(66);
                id.encode_impl(buffer);
            },
            ErrorKind::PartiallyInferedGenericType { id, r#type } => {
                buffer.push(67);
                id.encode_impl(buffer);
                r#type.encode_impl(buffer);
            },
            ErrorKind::CannotApplyInfixOp { op, arg_types } => {
                buffer.push(68);
                op.encode_impl(buffer);
                arg_types.encode_impl(buffer);
            },
            ErrorKind::MultipleModuleFiles { module, found_files } => {
                buffer.push(69);
                module.encode_impl(buffer);
                found_files.encode_impl(buffer);
            },
            ErrorKind::ModuleFileNotFound { module, candidates } => {
                buffer.push(70);
                module.encode_impl(buffer);
                candidates.encode_impl(buffer);
            },
            ErrorKind::LibFileNotFound => {
                buffer.push(71);
            },
            ErrorKind::UnusedNames { names, kind } => {
                buffer.push(72);
                names.encode_impl(buffer);
                kind.encode_impl(buffer);
            },
            ErrorKind::Todo { id, message } => {
                buffer.push(73);
                id.encode_impl(buffer);
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
            Some(21) => Ok((ErrorKind::MissingDocComment, cursor + 1)),
            Some(22) => Ok((ErrorKind::DocCommentNotAllowed, cursor + 1)),
            Some(23) => Ok((ErrorKind::ModuleDocCommentNotAtTop, cursor + 1)),
            Some(24) => {
                let (d, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::MissingDecorator(d), cursor))
            },
            Some(25) => Ok((ErrorKind::DecoratorNotAllowed, cursor + 1)),
            Some(26) => {
                let (d, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::UnexpectedDecorator(d), cursor))
            },
            Some(27) => Ok((ErrorKind::ModuleDecoratorNotAtTop, cursor + 1)),
            Some(28) => Ok((ErrorKind::MissingVisibility, cursor + 1)),
            Some(29) => Ok((ErrorKind::CannotBePublic, cursor + 1)),
            Some(30) => Ok((ErrorKind::BlockWithoutValue, cursor + 1)),
            Some(31) => Ok((ErrorKind::StructWithoutField, cursor + 1)),
            Some(32) => Ok((ErrorKind::EmptyCurlyBraceBlock, cursor + 1)),
            Some(33) => Ok((ErrorKind::PositionalArgAfterKeywordArg, cursor + 1)),
            Some(34) => Ok((ErrorKind::NonDefaultValueAfterDefaultValue, cursor + 1)),
            Some(35) => Ok((ErrorKind::CannotDeclareInlineModule, cursor + 1)),
            Some(36) => Ok((ErrorKind::InclusiveRangeWithNoEnd, cursor + 1)),
            Some(37) => Ok((ErrorKind::AstPatternTypeError, cursor + 1)),
            Some(38) => Ok((ErrorKind::DifferentNameBindingsInOrPattern, cursor + 1)),
            Some(39) => Ok((ErrorKind::InvalidFnType, cursor + 1)),
            Some(40) => Ok((ErrorKind::EmptyMatchStatement, cursor + 1)),
            Some(41) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::RedundantDecorator(s), cursor))
            },
            Some(42) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::InvalidDecorator(s), cursor))
            },
            Some(43) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CannotBindNameToAnotherName(s), cursor))
            },
            Some(44) => Ok((ErrorKind::CannotAnnotateType, cursor + 1)),
            Some(45) => {
                let (a, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                let (b, cursor) = InternedString::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::RedundantNameBinding(a, b), cursor))
            },
            Some(46) => {
                let (name, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::NameCollision { name }, cursor))
            },
            Some(47) => {
                let (names, cursor) = Vec::<InternedString>::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CyclicLet { names }, cursor))
            },
            Some(48) => {
                let (names, cursor) = Vec::<InternedString>::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CyclicAlias { names }, cursor))
            },
            Some(49) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::UndefinedName(s), cursor))
            },
            Some(50) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::KeywordArgumentRepeated(s), cursor))
            },
            Some(51) => Ok((ErrorKind::KeywordArgumentNotAllowed, cursor + 1)),
            Some(52) => Ok((ErrorKind::AliasResolveRecursionLimitReached, cursor + 1)),
            Some(53) => {
                let (expected, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::MissingTypeArgument { expected, got }, cursor))
            },
            Some(54) => {
                let (expected, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnexpectedTypeArgument { expected, got }, cursor))
            },
            Some(55) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::MissingKeywordArgument(s), cursor))
            },
            Some(56) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::InvalidKeywordArgument(s), cursor))
            },
            Some(57) => {
                let (expected, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::MissingArgument { expected, got }, cursor))
            },
            Some(58) => {
                let (expected, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnexpectedArgument { expected, got }, cursor))
            },
            Some(59) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::StructFieldRepeated(s), cursor))
            },
            Some(60) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::MissingStructField(s), cursor))
            },
            Some(61) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::InvalidStructField(s), cursor))
            },
            Some(62) => Ok((ErrorKind::DependentTypeNotAllowed, cursor + 1)),
            Some(63) => {
                let (expected, cursor) = String::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = String::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnexpectedType { expected, got }, cursor))
            },
            Some(64) => {
                let (id, cursor) = Option::<InternedString>::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CannotInferType { id }, cursor))
            },
            Some(65) => {
                let (id, cursor) = Option::<InternedString>::decode_impl(buffer, cursor + 1)?;
                let (r#type, cursor) = String::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::PartiallyInferedType { id, r#type }, cursor))
            },
            Some(66) => {
                let (id, cursor) = Option::<String>::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CannotInferGenericType { id }, cursor))
            },
            Some(67) => {
                let (id, cursor) = Option::<String>::decode_impl(buffer, cursor + 1)?;
                let (r#type, cursor) = String::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::PartiallyInferedGenericType { id, r#type }, cursor))
            },
            Some(68) => {
                let (op, cursor) = InfixOp::decode_impl(buffer, cursor + 1)?;
                let (arg_types, cursor) = Vec::<String>::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::CannotApplyInfixOp { op, arg_types }, cursor))
            },
            Some(69) => {
                let (module, cursor) = ModulePath::decode_impl(buffer, cursor + 1)?;
                let (found_files, cursor) = Vec::<String>::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::MultipleModuleFiles { module, found_files }, cursor))
            },
            Some(70) => {
                let (module, cursor) = ModulePath::decode_impl(buffer, cursor + 1)?;
                let (candidates, cursor) = Vec::<String>::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::ModuleFileNotFound { module, candidates }, cursor))
            },
            Some(71) => Ok((ErrorKind::LibFileNotFound, cursor + 1)),
            Some(72) => {
                let (names, cursor) = Vec::<InternedString>::decode_impl(buffer, cursor + 1)?;
                let (kind, cursor) = NameKind::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnusedNames { names, kind }, cursor))
            },
            Some(73) => {
                let (id, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                let (message, cursor) = String::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::Todo { id, message }, cursor))
            },
            Some(n @ 74..) => Err(DecodeError::InvalidEnumVariant(*n)),
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
            ErrorToken::Generic => {
                buffer.push(8);
            },
            ErrorToken::Number => {
                buffer.push(9);
            },
            ErrorToken::String => {
                buffer.push(10);
            },
            ErrorToken::TypeAnnotation => {
                buffer.push(11);
            },
            ErrorToken::Declaration => {
                buffer.push(12);
            },
            ErrorToken::Expr => {
                buffer.push(13);
            },
            ErrorToken::Block => {
                buffer.push(14);
            },
            ErrorToken::Operator => {
                buffer.push(15);
            },
            ErrorToken::AssignOrLt => {
                buffer.push(16);
            },
            ErrorToken::BraceOrCommaOrParenthesis => {
                buffer.push(17);
            },
            ErrorToken::BraceOrParenthesis => {
                buffer.push(18);
            },
            ErrorToken::ColonOrComma => {
                buffer.push(19);
            },
            ErrorToken::CommaOrDot => {
                buffer.push(20);
            },
            ErrorToken::CommaOrGt => {
                buffer.push(21);
            },
            ErrorToken::DotOrSemicolon => {
                buffer.push(22);
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
            Some(8) => Ok((ErrorToken::Generic, cursor + 1)),
            Some(9) => Ok((ErrorToken::Number, cursor + 1)),
            Some(10) => Ok((ErrorToken::String, cursor + 1)),
            Some(11) => Ok((ErrorToken::TypeAnnotation, cursor + 1)),
            Some(12) => Ok((ErrorToken::Declaration, cursor + 1)),
            Some(13) => Ok((ErrorToken::Expr, cursor + 1)),
            Some(14) => Ok((ErrorToken::Block, cursor + 1)),
            Some(15) => Ok((ErrorToken::Operator, cursor + 1)),
            Some(16) => Ok((ErrorToken::AssignOrLt, cursor + 1)),
            Some(17) => Ok((ErrorToken::BraceOrCommaOrParenthesis, cursor + 1)),
            Some(18) => Ok((ErrorToken::BraceOrParenthesis, cursor + 1)),
            Some(19) => Ok((ErrorToken::ColonOrComma, cursor + 1)),
            Some(20) => Ok((ErrorToken::CommaOrDot, cursor + 1)),
            Some(21) => Ok((ErrorToken::CommaOrGt, cursor + 1)),
            Some(22) => Ok((ErrorToken::DotOrSemicolon, cursor + 1)),
            Some(n @ 23..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
