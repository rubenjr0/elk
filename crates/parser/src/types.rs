use ast::types::{FunctionSignature, Type};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    combinator::{map, value},
    multi::separated_list1,
    sequence::delimited,
};

use crate::{
    common::{parse_identifier_upper, ws},
    custom_types::parse_custom_type_generics,
};

pub fn parse_type(input: &str) -> IResult<&str, Type> {
    alt((
        map(parse_function_signature, Type::Function),
        parse_primitive_type,
        map(parse_custom_type, |(name, generics)| {
            Type::Custom(name, generics)
        }),
    ))
    .parse(input)
}

fn parse_primitive_type(input: &str) -> IResult<&str, Type> {
    alt((
        value(Type::U8, tag("U8")),
        value(Type::I8, tag("I8")),
        value(Type::U16, tag("U16")),
        value(Type::I16, tag("I16")),
        value(Type::U32, tag("U32")),
        value(Type::I32, tag("I32")),
        value(Type::U64, tag("U64")),
        value(Type::I64, tag("I64")),
        value(Type::F32, tag("F32")),
        value(Type::F64, tag("F64")),
        value(Type::Bool, tag("Bool")),
        value(Type::String, tag("String")),
    ))
    .parse(input)
}

fn parse_custom_type(input: &str) -> IResult<&str, (String, Vec<String>)> {
    let (input, name) = parse_identifier_upper(input)?;
    let (input, generics) = parse_custom_type_generics(input)?;
    Ok((input, (name.to_owned(), generics)))
}

/// Types separated by arrows. The last type is the return type.
/// If parenthesis are encountered, its a function signature, parse recursively.
/// Example: `U8 -> U8 -> U8`
/// Example: `(U8 -> Bool) -> U8 -> Bool`
pub fn parse_function_signature(input: &str) -> IResult<&str, FunctionSignature> {
    let (remaining, args) = parse_function_args(input)?;
    let (return_type, args) = args.split_last().unwrap();
    if args.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            remaining,
            nom::error::ErrorKind::SeparatedList,
        )));
    }
    Ok((
        remaining,
        FunctionSignature::new(args.to_vec(), return_type.clone()),
    ))
}

fn parse_function_args(input: &str) -> IResult<&str, Vec<Type>> {
    let (remaining, args) = separated_list1(
        ws(tag("->")),
        alt((
            parse_primitive_type,
            map(parse_custom_type, |(name, generics)| {
                Type::Custom(name, generics)
            }),
            map(
                delimited(tag("("), parse_function_signature, tag(")")),
                Type::Function,
            ),
        )),
    )
    .parse(input)?;
    Ok((remaining, args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_type_u8() {
        let input = "U8";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::U8);
    }

    #[test]
    fn test_parse_type_i8() {
        let input = "I8";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::I8);
    }

    #[test]
    fn test_parse_type_bool() {
        let input = "Bool";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::Bool);
    }

    #[test]
    fn test_parse_type_f32() {
        let input = "F32";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::F32);
    }

    #[test]
    fn test_parse_type_f64() {
        let input = "F64";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::F64);
    }

    #[test]
    fn test_parse_type_string() {
        let input = "String";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::String);
    }

    #[test]
    fn test_parse_type_custom() {
        let input = "CustomType";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::Custom("CustomType".to_owned(), vec![]));
    }

    #[test]
    fn test_parse_type_custom_with_generics() {
        let input = "Option(T)";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(
            parsed,
            Type::Custom("Option".to_owned(), vec!["T".to_owned()])
        );
    }

    #[test]
    fn test_parse_simple_function_signature() {
        let input = "U8 -> U8";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(
            parsed,
            Type::Function(FunctionSignature::new(vec![Type::U8], Type::U8))
        );
    }

    #[test]
    fn test_parse_function_signature() {
        let input = "A -> B -> Bool";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(
            parsed,
            Type::Function(FunctionSignature::new(
                vec![
                    Type::Custom("A".to_owned(), vec![]),
                    Type::Custom("B".to_owned(), vec![])
                ],
                Type::Bool
            ))
        );
    }

    #[test]
    fn test_parse_higher_order_function_signature() {
        let input = "(U8 -> U8) -> U8 -> U8";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(
            parsed,
            Type::Function(FunctionSignature::new(
                vec![
                    Type::Function(FunctionSignature::new(vec![Type::U8], Type::U8)),
                    Type::U8
                ],
                Type::U8
            ))
        );
    }
}
