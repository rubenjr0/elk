use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, value},
    multi::separated_list1,
    sequence::delimited,
    IResult, Parser,
};

use crate::{
    parser::{parse_identifier_upper, ws},
    types::Type,
};

pub fn parse_type(input: &str) -> IResult<&str, Type> {
    alt((parse_function_type, parse_basic_type)).parse(input)
}

pub fn parse_basic_type(input: &str) -> IResult<&str, Type> {
    alt((
        value(Type::U8, tag("U8")),
        value(Type::I8, tag("I8")),
        value(Type::F32, tag("F32")),
        value(Type::F64, tag("F64")),
        value(Type::P8, tag("P8")),
        value(Type::P16, tag("P16")),
        value(Type::P32, tag("P32")),
        value(Type::Bool, tag("Bool")),
        value(Type::String, tag("String")),
        map(parse_identifier_upper, |s| Type::Custom(s.to_string())),
    ))
    .parse(input)
}

/// Types separated by arrows. The last type is the return type.
/// If parenthesis are encountered, its a function signature, parse recursively.
/// Example: `U8 -> U8 -> U8`
/// Example: `(U8 -> Bool) -> U8 -> Bool`
fn parse_function_type(input: &str) -> IResult<&str, Type> {
    let (remaining, args) = parse_function_args(input)?;
    let (return_type, args) = args.split_last().unwrap();

    Ok((
        remaining,
        match args {
            [] => return_type.to_owned(),
            _ => Type::FunctionSignature(args.to_vec(), Box::new(return_type.to_owned())),
        },
    ))
}

fn parse_function_args(input: &str) -> IResult<&str, Vec<Type>> {
    let (remaining, args) = separated_list1(
        ws(tag("->")),
        alt((
            parse_basic_type,
            delimited(tag("("), parse_function_type, tag(")")),
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
    fn test_parse_type_p8() {
        let input = "P8";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::P8);
    }

    #[test]
    fn test_parse_type_p16() {
        let input = "P16";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::P16);
    }

    #[test]
    fn test_parse_type_p32() {
        let input = "P32";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(parsed, Type::P32);
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
        assert_eq!(parsed, Type::Custom("CustomType".to_string()));
    }

    #[test]
    fn test_parse_simple_function_signature() {
        let input = "U8 -> U8";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(
            parsed,
            Type::FunctionSignature(vec![Type::U8], Box::new(Type::U8))
        );
    }

    #[test]
    fn test_parse_function_signature() {
        let input = "U8 -> U8 -> U8";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(
            parsed,
            Type::FunctionSignature(vec![Type::U8, Type::U8], Box::new(Type::U8))
        );
    }

    #[test]
    fn test_parse_higher_order_function_signature() {
        let input = "(U8 -> U8) -> U8 -> U8";
        let (_, parsed) = parse_type(input).unwrap();
        assert_eq!(
            parsed,
            Type::FunctionSignature(
                vec![
                    Type::FunctionSignature(vec![Type::U8], Box::new(Type::U8)),
                    Type::U8
                ],
                Box::new(Type::U8)
            )
        );
    }
}
