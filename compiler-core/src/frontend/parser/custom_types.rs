use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, terminated},
    IResult, Parser,
};

use crate::frontend::{
    ast::types::{
        custom::{CustomTypeContent, Field, Variant},
        CustomType, Type,
    },
    parser::common::{parse_identifier_upper, ws},
};

use super::{common::parse_identifier_lower, types::parse_type};

/// Custom types are defined as follows:
/// `type CustomType { VariantA, VariantB }`
pub fn parse_custom_type(input: &str) -> IResult<&str, CustomType> {
    let (input, _) = ws(tag("type")).parse(input)?;
    let (input, name) = ws(parse_identifier_upper).parse(input)?;
    let (input, generics) = parse_custom_type_generics(input)?;
    let (input, content) = delimited(
        ws(tag("{")),
        terminated(parse_custom_type_contents, opt(ws(tag(",")))),
        ws(tag("}")),
    )
    .or(|a| Ok((a, CustomTypeContent::Empty)))
    .parse(input)?;
    Ok((input, CustomType::new(name, content, generics)))
}

pub fn parse_custom_type_generics(input: &str) -> IResult<&str, Vec<String>> {
    opt(delimited(
        ws(tag("(")),
        separated_list1(ws(tag(",")), parse_identifier_upper.map(str::to_string)),
        ws(tag(")")),
    ))
    .map(Option::unwrap_or_default)
    .parse(input)
}

fn parse_custom_type_contents(input: &str) -> IResult<&str, CustomTypeContent> {
    alt((
        map(parse_variants, |vs| {
            CustomTypeContent::Enum(
                vs.into_iter()
                    .enumerate()
                    .map(|(i, v)| (i as u8, v))
                    .collect(),
            )
        }),
        map(parse_fields, |r| {
            let mut fields: Vec<Field> = r.into_iter().map(|(s, t)| Field::new(&s, t)).collect();
            fields.sort_by_key(|f| f.name().to_owned());
            CustomTypeContent::Record(fields)
        }),
    ))
    .parse(input)
}

fn parse_variants(input: &str) -> IResult<&str, Vec<Variant>> {
    let (remaining, variants) = separated_list1(ws(tag(",")), parse_variant).parse(input)?;
    Ok((remaining, variants))
}

fn parse_fields(input: &str) -> IResult<&str, Vec<(String, Type)>> {
    let (remaining, fields) = separated_list1(ws(tag(",")), parse_field).parse(input)?;
    Ok((remaining, fields))
}

fn parse_variant(input: &str) -> IResult<&str, Variant> {
    let (remaining, name) = parse_identifier_upper(input)?;
    let (remaining, variant) = alt((map(
        opt(delimited(
            tag("("),
            separated_list0(ws(tag(",")), parse_type),
            tag(")"),
        )),
        |types| Variant::new(name, types.unwrap_or_default()),
    ),))(remaining)?;

    Ok((remaining, variant))
}

fn parse_field(input: &str) -> IResult<&str, (String, Type)> {
    let (remaining, name) = parse_identifier_lower(input)?;
    let (remaining, _) = ws(tag(":")).parse(remaining)?;
    let (remaining, ty) = parse_type(remaining)?;
    Ok((remaining, (name.to_owned(), ty)))
}

#[cfg(test)]
mod tests {
    use crate::frontend::ast::types::custom::{CustomTypeContent, Field, Variant};
    use crate::frontend::ast::types::Type;

    #[test]
    fn test_parse_empty_custom_type() {
        let input = "type Phantom";
        let (_, parsed) = super::parse_custom_type(input).unwrap();
        assert_eq!(parsed.name(), "Phantom");
        assert_eq!(parsed.content(), &CustomTypeContent::Empty);
    }

    #[test]
    fn test_parse_custom_type_variants() {
        let input = "type CustomType { VariantA, VariantB, }";
        let (_, parsed) = super::parse_custom_type(input).unwrap();
        assert_eq!(parsed.name(), "CustomType");
        assert_eq!(
            parsed.content(),
            &CustomTypeContent::Enum(vec![
                (0, Variant::new("VariantA", vec![])),
                (1, Variant::new("VariantB", vec![])),
            ])
        );
    }

    #[test]
    fn test_parse_custom_type_generics() {
        let input = "type Option(T) { Some(T), None }";
        let (_, parsed) = super::parse_custom_type(input).unwrap();
        assert_eq!(parsed.name(), "Option");
        assert_eq!(
            parsed.content(),
            &CustomTypeContent::Enum(vec![
                (
                    0,
                    Variant::new("Some", vec![Type::Custom("T".to_owned(), vec![])])
                ),
                (1, Variant::new("None", vec![])),
            ])
        );
    }

    #[test]
    fn test_parse_custom_type_record() {
        let input = "type CustomType { admin: Bool, age: U8, }";
        let (_, parsed) = super::parse_custom_type(input).unwrap();
        assert_eq!(parsed.name(), "CustomType");
        assert_eq!(
            parsed.content(),
            &CustomTypeContent::Record(vec![
                Field::new("admin", Type::Bool),
                Field::new("age", Type::U8),
            ])
        );
    }

    #[test]
    fn test_parse_variants() {
        let input = "VariantA, VariantB";
        let (_, parsed) = super::parse_variants(input).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_parse_variants_fails() {
        let input = "admin: Bool, age: U8";
        let res = super::parse_variants(input);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_named_variant() {
        let input = "VariantA";
        let (_, parsed) = super::parse_variant(input).unwrap();
        assert_eq!(parsed.name(), input);
        assert!(parsed.types().is_empty());
    }

    #[test]
    fn test_parse_tuple_variant() {
        let input = "VariantA(U8)";
        let (_, parsed) = super::parse_variant(input).unwrap();
        assert_eq!(parsed.name(), "VariantA");
        assert_eq!(parsed.types(), &vec![Type::U8]);
    }

    #[test]
    fn test_parse_fields() {
        let input = "admin: Bool, age: U8";
        let (_, parsed) = super::parse_fields(input).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_parse_fields_fails() {
        let input = "VariantA, VariantB";
        let res = super::parse_fields(input);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_field() {
        let input = "admin: Bool";
        let (_, parsed) = super::parse_field(input).unwrap();
        assert_eq!(parsed.0, "admin");
        assert_eq!(parsed.1, Type::Bool);
    }
}
