use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, value},
    multi::separated_list1,
    sequence::delimited,
    IResult, Parser,
};

use crate::ast::types::{FunctionSignature, PrimitiveType, Type};

use super::{
    common::{parse_identifier_upper, ws},
    custom_types::parse_custom_type_generics,
};

pub fn parse_type(input: &str) -> IResult<&str, Type> {
    alt((
        map(parse_function_signature, Type::Function),
        map(parse_primitive_type, Type::Primitive),
        map(parse_custom_type, |(name, generics)| {
            Type::Custom(name, generics)
        }),
    ))
    .parse(input)
}

fn parse_primitive_type(input: &str) -> IResult<&str, PrimitiveType> {
    alt((
        value(PrimitiveType::U8, tag("U8")),
        value(PrimitiveType::I8, tag("I8")),
        value(PrimitiveType::F32, tag("F32")),
        value(PrimitiveType::F64, tag("F64")),
        value(PrimitiveType::P8, tag("P8")),
        value(PrimitiveType::P16, tag("P16")),
        value(PrimitiveType::P32, tag("P32")),
        value(PrimitiveType::Bool, tag("Bool")),
        value(PrimitiveType::String, tag("String")),
    ))
    .parse(input)
}

fn parse_custom_type(input: &str) -> IResult<&str, (String, Vec<String>)> {
    let (input, name) = parse_identifier_upper(input)?;
    let (input, generics) = parse_custom_type_generics(input)?;
    Ok((input, (name.to_string(), generics)))
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
            map(parse_primitive_type, Type::Primitive),
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
        assert_eq!(parsed, Type::Primitive(PrimitiveType::U8));
    }

    #[test]
    fn test_parse_type_i8() {
        let input = "I8";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::Primitive(PrimitiveType::I8));
    }

    #[test]
    fn test_parse_type_bool() {
        let input = "Bool";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::Primitive(PrimitiveType::Bool));
    }

    #[test]
    fn test_parse_type_f32() {
        let input = "F32";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::Primitive(PrimitiveType::F32));
    }

    #[test]
    fn test_parse_type_f64() {
        let input = "F64";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::Primitive(PrimitiveType::F64));
    }

    #[test]
    fn test_parse_type_p8() {
        let input = "P8";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::Primitive(PrimitiveType::P8));
    }

    #[test]
    fn test_parse_type_p16() {
        let input = "P16";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::Primitive(PrimitiveType::P16));
    }

    #[test]
    fn test_parse_type_p32() {
        let input = "P32";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::Primitive(PrimitiveType::P32));
    }

    #[test]
    fn test_parse_type_string() {
        let input = "String";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::Primitive(PrimitiveType::String));
    }

    #[test]
    fn test_parse_type_custom() {
        let input = "CustomType";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::Custom("CustomType".to_string(), vec![]));
    }

    #[test]
    fn test_parse_type_custom_with_generics() {
        let input = "Option(T)";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(
            parsed,
            Type::Custom("Option".to_string(), vec!["T".to_string()])
        );
    }

    #[test]
    fn test_parse_simple_function_signature() {
        let input = "U8 -> U8";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(
            parsed,
            Type::Function(FunctionSignature::new(
                vec![Type::Primitive(PrimitiveType::U8)],
                Type::Primitive(PrimitiveType::U8)
            ))
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
                    Type::Custom("A".to_string(), vec![]),
                    Type::Custom("B".to_string(), vec![])
                ],
                Type::Primitive(PrimitiveType::Bool)
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
                    Type::Function(FunctionSignature::new(
                        vec![Type::Primitive(PrimitiveType::U8)],
                        Type::Primitive(PrimitiveType::U8)
                    )),
                    Type::Primitive(PrimitiveType::U8)
                ],
                Type::Primitive(PrimitiveType::U8)
            ))
        );
    }
}
