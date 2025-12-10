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
            ErrorKind::NotAllowedCharInFString(ch) => {
                buffer.push(5);
                ch.encode_impl(buffer);
            },
            ErrorKind::InvalidCharLiteral => {
                buffer.push(6);
            },
            ErrorKind::InvalidCharLiteralPrefix => {
                buffer.push(7);
            },
            ErrorKind::UnterminatedCharLiteral => {
                buffer.push(8);
            },
            ErrorKind::InvalidByteLiteral => {
                buffer.push(9);
            },
            ErrorKind::InvalidEscape => {
                buffer.push(10);
            },
            ErrorKind::EmptyCharLiteral => {
                buffer.push(11);
            },
            ErrorKind::UnterminatedBlockComment => {
                buffer.push(12);
            },
            ErrorKind::InvalidUtf8 => {
                buffer.push(13);
            },
            ErrorKind::InvalidUnicodeCharacter => {
                buffer.push(14);
            },
            ErrorKind::InvalidUnicodeEscape => {
                buffer.push(15);
            },
            ErrorKind::UnmatchedGroup { expected, got } => {
                buffer.push(16);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::TooManyQuotes => {
                buffer.push(17);
            },
            ErrorKind::UnclosedDelimiter(delim) => {
                buffer.push(18);
                delim.encode_impl(buffer);
            },
            ErrorKind::UnexpectedToken { expected, got } => {
                buffer.push(19);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::UnexpectedEof { expected } => {
                buffer.push(20);
                expected.encode_impl(buffer);
            },
            ErrorKind::UnexpectedEog { expected } => {
                buffer.push(21);
                expected.encode_impl(buffer);
            },
            ErrorKind::MissingDocComment => {
                buffer.push(22);
            },
            ErrorKind::DocCommentNotAllowed => {
                buffer.push(23);
            },
            ErrorKind::ModuleDocCommentNotAtTop => {
                buffer.push(24);
            },
            ErrorKind::MissingDecorator(d) => {
                buffer.push(25);
                d.encode_impl(buffer);
            },
            ErrorKind::DecoratorNotAllowed => {
                buffer.push(26);
            },
            ErrorKind::UnexpectedDecorator(d) => {
                buffer.push(27);
                d.encode_impl(buffer);
            },
            ErrorKind::ModuleDecoratorNotAtTop => {
                buffer.push(28);
            },
            ErrorKind::MissingVisibility => {
                buffer.push(29);
            },
            ErrorKind::CannotBePublic => {
                buffer.push(30);
            },
            ErrorKind::FunctionWithoutBody => {
                buffer.push(31);
            },
            ErrorKind::BlockWithoutValue => {
                buffer.push(32);
            },
            ErrorKind::StructWithoutField => {
                buffer.push(33);
            },
            ErrorKind::EmptyCurlyBraceBlock => {
                buffer.push(34);
            },
            ErrorKind::PositionalArgAfterKeywordArg => {
                buffer.push(35);
            },
            ErrorKind::NonDefaultValueAfterDefaultValue => {
                buffer.push(36);
            },
            ErrorKind::CannotDeclareInlineModule => {
                buffer.push(37);
            },
            ErrorKind::InclusiveRangeWithNoEnd => {
                buffer.push(38);
            },
            ErrorKind::DotDotDotDot => {
                buffer.push(39);
            },
            ErrorKind::DifferentNameBindingsInOrPattern => {
                buffer.push(40);
            },
            ErrorKind::InvalidFnType => {
                buffer.push(41);
            },
            ErrorKind::EmptyMatchStatement => {
                buffer.push(42);
            },
            ErrorKind::RedundantDecorator(s) => {
                buffer.push(43);
                s.encode_impl(buffer);
            },
            ErrorKind::InvalidDecorator(s) => {
                buffer.push(44);
                s.encode_impl(buffer);
            },
            ErrorKind::MissingDecoratorArgument { expected, got } => {
                buffer.push(45);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::UnexpectedDecoratorArgument { expected, got } => {
                buffer.push(46);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::CannotBindNameToAnotherName(s) => {
                buffer.push(47);
                s.encode_impl(buffer);
            },
            ErrorKind::CannotAnnotateType => {
                buffer.push(48);
            },
            ErrorKind::RedundantNameBinding(a, b) => {
                buffer.push(49);
                a.encode_impl(buffer);
                b.encode_impl(buffer);
            },
            ErrorKind::NameCollision { name } => {
                buffer.push(50);
                name.encode_impl(buffer);
            },
            ErrorKind::CyclicLet { names } => {
                buffer.push(51);
                names.encode_impl(buffer);
            },
            ErrorKind::CyclicAlias { names } => {
                buffer.push(52);
                names.encode_impl(buffer);
            },
            ErrorKind::UndefinedName(s) => {
                buffer.push(53);
                s.encode_impl(buffer);
            },
            ErrorKind::KeywordArgumentRepeated(s) => {
                buffer.push(54);
                s.encode_impl(buffer);
            },
            ErrorKind::KeywordArgumentNotAllowed => {
                buffer.push(55);
            },
            ErrorKind::AliasResolveRecursionLimitReached => {
                buffer.push(56);
            },
            ErrorKind::MissingTypeParameter { expected, got } => {
                buffer.push(57);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::UnexpectedTypeParameter { expected, got } => {
                buffer.push(58);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::MissingKeywordArgument(s) => {
                buffer.push(59);
                s.encode_impl(buffer);
            },
            ErrorKind::InvalidKeywordArgument(s) => {
                buffer.push(60);
                s.encode_impl(buffer);
            },
            ErrorKind::MissingFunctionParameter { expected, got } => {
                buffer.push(61);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::UnexpectedFunctionParameter { expected, got } => {
                buffer.push(62);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::StructFieldRepeated(s) => {
                buffer.push(63);
                s.encode_impl(buffer);
            },
            ErrorKind::MissingStructField(s) => {
                buffer.push(64);
                s.encode_impl(buffer);
            },
            ErrorKind::InvalidStructField(s) => {
                buffer.push(65);
                s.encode_impl(buffer);
            },
            ErrorKind::DependentTypeNotAllowed => {
                buffer.push(66);
            },
            ErrorKind::UnexpectedType { expected, got } => {
                buffer.push(67);
                expected.encode_impl(buffer);
                got.encode_impl(buffer);
            },
            ErrorKind::CannotInferType { id } => {
                buffer.push(68);
                id.encode_impl(buffer);
            },
            ErrorKind::PartiallyInferedType { id, r#type } => {
                buffer.push(69);
                id.encode_impl(buffer);
                r#type.encode_impl(buffer);
            },
            ErrorKind::CannotInferGenericType { id } => {
                buffer.push(70);
                id.encode_impl(buffer);
            },
            ErrorKind::PartiallyInferedGenericType { id, r#type } => {
                buffer.push(71);
                id.encode_impl(buffer);
                r#type.encode_impl(buffer);
            },
            ErrorKind::CannotApplyInfixOp { op, arg_types } => {
                buffer.push(72);
                op.encode_impl(buffer);
                arg_types.encode_impl(buffer);
            },
            ErrorKind::CannotSpecializePolyGeneric { num_candidates } => {
                buffer.push(73);
                num_candidates.encode_impl(buffer);
            },
            ErrorKind::MultipleModuleFiles { module, found_files } => {
                buffer.push(74);
                module.encode_impl(buffer);
                found_files.encode_impl(buffer);
            },
            ErrorKind::ModuleFileNotFound { module, candidates } => {
                buffer.push(75);
                module.encode_impl(buffer);
                candidates.encode_impl(buffer);
            },
            ErrorKind::LibFileNotFound => {
                buffer.push(76);
            },
            ErrorKind::UnusedNames { names, kind } => {
                buffer.push(77);
                names.encode_impl(buffer);
                kind.encode_impl(buffer);
            },
            ErrorKind::Todo { id, message } => {
                buffer.push(78);
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
            Some(6) => {
                let (ch, cursor) = u8::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::NotAllowedCharInFString(ch), cursor))
            },
            Some(7) => Ok((ErrorKind::InvalidCharLiteralPrefix, cursor + 1)),
            Some(8) => Ok((ErrorKind::UnterminatedCharLiteral, cursor + 1)),
            Some(9) => Ok((ErrorKind::InvalidByteLiteral, cursor + 1)),
            Some(10) => Ok((ErrorKind::InvalidEscape, cursor + 1)),
            Some(11) => Ok((ErrorKind::EmptyCharLiteral, cursor + 1)),
            Some(12) => Ok((ErrorKind::UnterminatedBlockComment, cursor + 1)),
            Some(13) => Ok((ErrorKind::InvalidUtf8, cursor + 1)),
            Some(14) => Ok((ErrorKind::InvalidUnicodeCharacter, cursor + 1)),
            Some(15) => Ok((ErrorKind::InvalidUnicodeEscape, cursor + 1)),
            Some(16) => {
                let (expected, cursor) = u8::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = u8::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnmatchedGroup { expected, got }, cursor))
            },
            Some(17) => Ok((ErrorKind::TooManyQuotes, cursor + 1)),
            Some(18) => {
                let (delim, cursor) = u8::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::UnclosedDelimiter(delim), cursor))
            },
            Some(19) => {
                let (expected, cursor) = ErrorToken::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = ErrorToken::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnexpectedToken { expected, got }, cursor))
            },
            Some(20) => {
                let (expected, cursor) = ErrorToken::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::UnexpectedEof { expected }, cursor))
            },
            Some(21) => {
                let (expected, cursor) = ErrorToken::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::UnexpectedEog { expected }, cursor))
            },
            Some(22) => Ok((ErrorKind::MissingDocComment, cursor + 1)),
            Some(23) => Ok((ErrorKind::DocCommentNotAllowed, cursor + 1)),
            Some(24) => Ok((ErrorKind::ModuleDocCommentNotAtTop, cursor + 1)),
            Some(25) => {
                let (d, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::MissingDecorator(d), cursor))
            },
            Some(26) => Ok((ErrorKind::DecoratorNotAllowed, cursor + 1)),
            Some(27) => {
                let (d, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::UnexpectedDecorator(d), cursor))
            },
            Some(28) => Ok((ErrorKind::ModuleDecoratorNotAtTop, cursor + 1)),
            Some(29) => Ok((ErrorKind::MissingVisibility, cursor + 1)),
            Some(30) => Ok((ErrorKind::CannotBePublic, cursor + 1)),
            Some(31) => Ok((ErrorKind::FunctionWithoutBody, cursor + 1)),
            Some(32) => Ok((ErrorKind::BlockWithoutValue, cursor + 1)),
            Some(33) => Ok((ErrorKind::StructWithoutField, cursor + 1)),
            Some(34) => Ok((ErrorKind::EmptyCurlyBraceBlock, cursor + 1)),
            Some(35) => Ok((ErrorKind::PositionalArgAfterKeywordArg, cursor + 1)),
            Some(36) => Ok((ErrorKind::NonDefaultValueAfterDefaultValue, cursor + 1)),
            Some(37) => Ok((ErrorKind::CannotDeclareInlineModule, cursor + 1)),
            Some(38) => Ok((ErrorKind::InclusiveRangeWithNoEnd, cursor + 1)),
            Some(39) => Ok((ErrorKind::DotDotDotDot, cursor + 1)),
            Some(40) => Ok((ErrorKind::DifferentNameBindingsInOrPattern, cursor + 1)),
            Some(41) => Ok((ErrorKind::InvalidFnType, cursor + 1)),
            Some(42) => Ok((ErrorKind::EmptyMatchStatement, cursor + 1)),
            Some(43) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::RedundantDecorator(s), cursor))
            },
            Some(44) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::InvalidDecorator(s), cursor))
            },
            Some(45) => {
                let (expected, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::MissingDecoratorArgument { expected, got }, cursor))
            },
            Some(46) => {
                let (expected, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnexpectedDecoratorArgument { expected, got }, cursor))
            },
            Some(47) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CannotBindNameToAnotherName(s), cursor))
            },
            Some(48) => Ok((ErrorKind::CannotAnnotateType, cursor + 1)),
            Some(49) => {
                let (a, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                let (b, cursor) = InternedString::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::RedundantNameBinding(a, b), cursor))
            },
            Some(50) => {
                let (name, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::NameCollision { name }, cursor))
            },
            Some(51) => {
                let (names, cursor) = Vec::<InternedString>::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CyclicLet { names }, cursor))
            },
            Some(52) => {
                let (names, cursor) = Vec::<InternedString>::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CyclicAlias { names }, cursor))
            },
            Some(53) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::UndefinedName(s), cursor))
            },
            Some(54) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::KeywordArgumentRepeated(s), cursor))
            },
            Some(55) => Ok((ErrorKind::KeywordArgumentNotAllowed, cursor + 1)),
            Some(56) => Ok((ErrorKind::AliasResolveRecursionLimitReached, cursor + 1)),
            Some(57) => {
                let (expected, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::MissingTypeParameter { expected, got }, cursor))
            },
            Some(58) => {
                let (expected, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnexpectedTypeParameter { expected, got }, cursor))
            },
            Some(59) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::MissingKeywordArgument(s), cursor))
            },
            Some(60) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::InvalidKeywordArgument(s), cursor))
            },
            Some(61) => {
                let (expected, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::MissingFunctionParameter { expected, got }, cursor))
            },
            Some(62) => {
                let (expected, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnexpectedFunctionParameter { expected, got }, cursor))
            },
            Some(63) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::StructFieldRepeated(s), cursor))
            },
            Some(64) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::MissingStructField(s), cursor))
            },
            Some(65) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::InvalidStructField(s), cursor))
            },
            Some(66) => Ok((ErrorKind::DependentTypeNotAllowed, cursor + 1)),
            Some(67) => {
                let (expected, cursor) = String::decode_impl(buffer, cursor + 1)?;
                let (got, cursor) = String::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnexpectedType { expected, got }, cursor))
            },
            Some(68) => {
                let (id, cursor) = Option::<InternedString>::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CannotInferType { id }, cursor))
            },
            Some(69) => {
                let (id, cursor) = Option::<InternedString>::decode_impl(buffer, cursor + 1)?;
                let (r#type, cursor) = String::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::PartiallyInferedType { id, r#type }, cursor))
            },
            Some(70) => {
                let (id, cursor) = Option::<String>::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CannotInferGenericType { id }, cursor))
            },
            Some(71) => {
                let (id, cursor) = Option::<String>::decode_impl(buffer, cursor + 1)?;
                let (r#type, cursor) = String::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::PartiallyInferedGenericType { id, r#type }, cursor))
            },
            Some(72) => {
                let (op, cursor) = InfixOp::decode_impl(buffer, cursor + 1)?;
                let (arg_types, cursor) = Vec::<String>::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::CannotApplyInfixOp { op, arg_types }, cursor))
            },
            Some(73) => {
                let (num_candidates, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorKind::CannotSpecializePolyGeneric { num_candidates }, cursor))
            },
            Some(74) => {
                let (module, cursor) = ModulePath::decode_impl(buffer, cursor + 1)?;
                let (found_files, cursor) = Vec::<String>::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::MultipleModuleFiles { module, found_files }, cursor))
            },
            Some(75) => {
                let (module, cursor) = ModulePath::decode_impl(buffer, cursor + 1)?;
                let (candidates, cursor) = Vec::<String>::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::ModuleFileNotFound { module, candidates }, cursor))
            },
            Some(76) => Ok((ErrorKind::LibFileNotFound, cursor + 1)),
            Some(77) => {
                let (names, cursor) = Vec::<InternedString>::decode_impl(buffer, cursor + 1)?;
                let (kind, cursor) = NameKind::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::UnusedNames { names, kind }, cursor))
            },
            Some(78) => {
                let (id, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                let (message, cursor) = String::decode_impl(buffer, cursor)?;
                Ok((ErrorKind::Todo { id, message }, cursor))
            },
            Some(n @ 79..) => Err(DecodeError::InvalidEnumVariant(*n)),
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
            ErrorToken::Path => {
                buffer.push(14);
            },
            ErrorToken::Pattern => {
                buffer.push(15);
            },
            ErrorToken::Item => {
                buffer.push(16);
            },
            ErrorToken::Block => {
                buffer.push(17);
            },
            ErrorToken::Operator => {
                buffer.push(18);
            },
            ErrorToken::AssignOrLt => {
                buffer.push(19);
            },
            ErrorToken::AssignOrSemicolon => {
                buffer.push(20);
            },
            ErrorToken::BraceOrCommaOrParenthesis => {
                buffer.push(21);
            },
            ErrorToken::BraceOrParenthesis => {
                buffer.push(22);
            },
            ErrorToken::ColonOrComma => {
                buffer.push(23);
            },
            ErrorToken::CommaOrDot => {
                buffer.push(24);
            },
            ErrorToken::CommaOrGt => {
                buffer.push(25);
            },
            ErrorToken::DotOrSemicolon => {
                buffer.push(26);
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
            Some(14) => Ok((ErrorToken::Path, cursor + 1)),
            Some(15) => Ok((ErrorToken::Pattern, cursor + 1)),
            Some(16) => Ok((ErrorToken::Item, cursor + 1)),
            Some(17) => Ok((ErrorToken::Block, cursor + 1)),
            Some(18) => Ok((ErrorToken::Operator, cursor + 1)),
            Some(19) => Ok((ErrorToken::AssignOrLt, cursor + 1)),
            Some(20) => Ok((ErrorToken::AssignOrSemicolon, cursor + 1)),
            Some(21) => Ok((ErrorToken::BraceOrCommaOrParenthesis, cursor + 1)),
            Some(22) => Ok((ErrorToken::BraceOrParenthesis, cursor + 1)),
            Some(23) => Ok((ErrorToken::ColonOrComma, cursor + 1)),
            Some(24) => Ok((ErrorToken::CommaOrDot, cursor + 1)),
            Some(25) => Ok((ErrorToken::CommaOrGt, cursor + 1)),
            Some(26) => Ok((ErrorToken::DotOrSemicolon, cursor + 1)),
            Some(n @ 27..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
