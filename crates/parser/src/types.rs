use crate::custom_types::parse_custom_type_generics;
use crate::functions::parse_function_signature;
use crate::identifiers::parse_identifier_upper;
use ast::types::Type;
use winnow::combinator::{alt, opt};
use winnow::{Parser, Result, ascii::alphanumeric1};

pub fn parse_type(input: &mut &str) -> Result<Type> {
    alt((
        parse_function_signature.map(Type::Function),
        parse_primitive_type,
        parse_custom_type,
    ))
    .parse_next(input)
}

pub fn parse_primitive_type(input: &mut &str) -> Result<Type> {
    alphanumeric1
        .verify_map(|s: &str| match s {
            "I8" => Some(Type::I8),
            "I16" => Some(Type::I16),
            "I32" => Some(Type::I32),
            "I64" => Some(Type::I64),
            "U8" => Some(Type::U8),
            "U16" => Some(Type::U16),
            "U32" => Some(Type::U32),
            "U64" => Some(Type::U64),
            "F32" => Some(Type::F32),
            "F64" => Some(Type::F64),
            "Bool" => Some(Type::Bool),
            "String" => Some(Type::String),
            "Unit" => Some(Type::Unit),
            _ => None,
        })
        .parse_next(input)
}

pub fn parse_custom_type(input: &mut &str) -> Result<Type> {
    let name = parse_identifier_upper.parse_next(input)?;
    let generics = opt(parse_custom_type_generics)
        .parse_next(input)?
        .unwrap_or_default();
    Ok(Type::Custom(name.to_owned(), generics))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_primitive_type() {
        let mut input = "U8";

        let (_, parsed) = parse_primitive_type.parse_peek(input).unwrap();
        assert_eq!(parsed, Type::U8);

        let parsed = parse_type(&mut input).unwrap();
        assert_eq!(parsed, Type::U8);
    }

    #[test]
    fn test_parse_type_custom() {
        let mut input = "CustomType";
        let expected = Type::Custom("CustomType".to_owned(), vec![]);

        let (_, parsed) = parse_custom_type.parse_peek(input).unwrap();
        assert_eq!(parsed, expected);

        let parsed = parse_type(&mut input).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_type_custom_with_generics() {
        let mut input = "Option<A>";
        let expected = Type::Custom("Option".to_owned(), vec!["A".to_owned()]);

        let (_, parsed) = parse_custom_type.parse_peek(input).unwrap();
        assert_eq!(parsed, expected);

        let parsed = parse_type(&mut input).unwrap();
        assert_eq!(parsed, expected);
    }
}
