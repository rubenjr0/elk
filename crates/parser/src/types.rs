use crate::custom_types::parse_custom_type_generics;
use crate::identifiers::parse_identifier_upper;
use crate::ws;
use ast::types::{FunctionSignature, Type};
use winnow::combinator::{alt, delimited, opt, separated, separated_pair};
use winnow::{Parser, Result, ascii::alphanumeric1};

pub fn parse_type(input: &mut &str) -> Result<Type> {
    alt((
        parse_function_signature,
        parse_primitive_type,
        parse_custom_type,
    ))
    .parse_next(input)
}

fn parse_function_signature(input: &mut &str) -> Result<Type> {
    let (args, out) = separated_pair(
        separated(
            1..,
            alt((
                parse_primitive_type,
                parse_custom_type,
                delimited('(', parse_function_signature, ')'),
            )),
            ws(','),
        ),
        ws("->"),
        parse_type,
    )
    .parse_next(input)?;

    Ok(Type::Function(FunctionSignature::new(args, out)))
}

fn parse_primitive_type(input: &mut &str) -> Result<Type> {
    alphanumeric1.parse_to().parse_next(input)
}

fn parse_custom_type(input: &mut &str) -> Result<Type> {
    let name = parse_identifier_upper.parse_next(input)?;
    let generics = opt(parse_custom_type_generics)
        .parse_next(input)?
        .unwrap_or_default();
    Ok(Type::Custom(name.to_owned(), generics))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::types::FunctionSignature;

    #[test]
    fn test_parse_primitive_type() {
        let mut input = "U8";

        let (_, parsed) = parse_primitive_type.parse_peek(&mut input).unwrap();
        assert_eq!(parsed, Type::U8);

        let parsed = parse_type(&mut input).unwrap();
        assert_eq!(parsed, Type::U8);
    }

    #[test]
    fn test_parse_type_custom() {
        let mut input = "CustomType";
        let expected = Type::Custom("CustomType".to_owned(), vec![]);

        let (_, parsed) = parse_custom_type.parse_peek(&mut input).unwrap();
        assert_eq!(parsed, expected);

        let parsed = parse_type(&mut input).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_type_custom_with_generics() {
        let mut input = "Option(T)";
        let expected = Type::Custom("Option".to_owned(), vec!["T".to_owned()]);

        let (_, parsed) = parse_custom_type.parse_peek(&mut input).unwrap();
        assert_eq!(parsed, expected);

        let parsed = parse_type(&mut input).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_function_signature() {
        let mut input = "A, U8 -> Bool";
        let expected = Type::Function(FunctionSignature::new(
            vec![Type::Custom("A".to_owned(), vec![]), Type::U8],
            Type::Bool,
        ));

        let (_, parsed) = parse_function_signature.parse_peek(&mut input).unwrap();
        assert_eq!(parsed, expected);

        let parsed = parse_type(&mut input).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_higher_order_function_signature() {
        let mut input = "(U8 -> U8), U8 -> U8";
        let expected = Type::Function(FunctionSignature::new(
            vec![
                Type::Function(FunctionSignature::new(vec![Type::U8], Type::U8)),
                Type::U8,
            ],
            Type::U8,
        ));

        let (_, parsed) = parse_function_signature.parse_peek(&mut input).unwrap();
        assert_eq!(parsed, expected);

        let parsed = parse_type(&mut input).unwrap();
        assert_eq!(parsed, expected);
    }
}
