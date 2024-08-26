use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    multi::{separated_list0, separated_list1},
    sequence::delimited,
    IResult, Parser,
};

use crate::{
    ast::types::{
        custom::{CustomTypeContent, Variant},
        CustomType, Type,
    },
    parser::common::{parse_identifier_upper, ws},
};

use super::{common::parse_identifier_lower, types::parse_type};

/// Custom types are defined as follows:
/// `type CustomType { VariantA, VariantB }`
pub fn parse_custom_type(input: &str) -> IResult<&str, CustomType> {
    let (remaining, _) = ws(tag("type")).parse(input)?;
    let (remaining, name) = ws(parse_identifier_upper).parse(remaining)?;
    let (remaining, content) =
        delimited(ws(tag("{")), parse_custom_type_contents, ws(tag("}"))).parse(remaining)?;

    Ok((remaining, CustomType::new(name, content)))
}

fn parse_custom_type_contents(input: &str) -> IResult<&str, CustomTypeContent> {
    if let Ok((remaining, contents)) = alt((
        map(parse_variants, CustomTypeContent::Variants),
        map(parse_fields, CustomTypeContent::Fields),
    ))
    .parse(input)
    {
        Ok((remaining, contents))
    } else {
        Ok((input, CustomTypeContent::Empty))
    }
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
    let (remaining, variant) = alt((
        map(
            delimited(
                tag("("),
                separated_list0(ws(tag(",")), parse_type),
                tag(")"),
            ),
            |types| Variant::new(name, types),
        ),
        map(tag(""), |_| Variant::named(name)), // can this be simplified?
    ))(remaining)?;

    Ok((remaining, variant))
}

fn parse_field(input: &str) -> IResult<&str, (String, Type)> {
    let (remaining, name) = parse_identifier_lower(input)?;
    let (remaining, _) = ws(tag(":")).parse(remaining)?;
    let (remaining, ty) = parse_type(remaining)?;
    Ok((remaining, (name.to_string(), ty)))
}

#[cfg(test)]
mod tests {
    use crate::ast::types::custom::{CustomTypeContent, Variant};
    use crate::ast::types::{PrimitiveType, Type};

    #[test]
    fn test_parse_empty_custom_type() {
        let input = "type Phantom {}";
        let (_, parsed) = super::parse_custom_type(input).unwrap();
        assert_eq!(parsed.name(), "Phantom");
        assert_eq!(parsed.content(), &CustomTypeContent::Empty);
    }

    #[test]
    fn test_parse_custom_type_variants() {
        let input = "type CustomType { VariantA, VariantB }";
        let (_, parsed) = super::parse_custom_type(input).unwrap();
        assert_eq!(parsed.name(), "CustomType");
        assert_eq!(
            parsed.content(),
            &CustomTypeContent::Variants(vec![
                Variant::named("VariantA"),
                Variant::named("VariantB"),
            ])
        );
    }

    #[test]
    fn test_parse_custom_type_record() {
        let input = "type CustomType { admin: Bool, age: U8 }";
        let (_, parsed) = super::parse_custom_type(input).unwrap();
        assert_eq!(parsed.name(), "CustomType");
        assert_eq!(
            parsed.content(),
            &CustomTypeContent::Fields(vec![
                ("admin".to_string(), Type::Primitive(PrimitiveType::Bool)),
                ("age".to_string(), Type::Primitive(PrimitiveType::U8)),
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
        assert_eq!(parsed.types(), &vec![Type::Primitive(PrimitiveType::U8)]);
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
        assert_eq!(parsed.1, Type::Primitive(PrimitiveType::Bool));
    }
}
