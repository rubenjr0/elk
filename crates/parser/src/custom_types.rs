use ast::types::{
    CustomType, Type,
    custom::{CustomTypeContent, Field, Variant},
};
use winnow::{
    Parser, Result,
    combinator::{alt, delimited, opt, separated, terminated},
};

use crate::{
    identifiers::{parse_identifier_lower, parse_identifier_upper},
    keyword,
    types::parse_type,
    ws,
};

/// Custom types are defined as follows:
/// `type CustomType { VariantA, VariantB }`
pub fn parse_custom_type_definition(input: &mut &str) -> Result<CustomType> {
    let _ = ws(keyword("type")).parse_next(input)?;
    let name = ws(parse_identifier_upper).parse_next(input)?;
    let generics = opt(parse_custom_type_generics)
        .parse_next(input)?
        .unwrap_or_default();
    let content = opt(delimited(
        ws('{'),
        terminated(parse_custom_type_contents, opt(ws(','))),
        ws('}'),
    ))
    .parse_next(input)?;
    Ok(CustomType::new(name, content, generics))
}

/// Generic parameters/arguments in angle brackets: `<A>`, `<A, B>`.
/// Used for both type definitions (`type Option<A>`), type references
/// (`Option<A>`), and function type-variable definitions (`map<A, B>(...)`).
pub fn parse_custom_type_generics(input: &mut &str) -> Result<Vec<String>> {
    delimited(
        '<',
        separated(1.., parse_identifier_upper.map(ToOwned::to_owned), ws(',')),
        '>',
    )
    .parse_next(input)
}

fn parse_custom_type_contents(input: &mut &str) -> Result<CustomTypeContent> {
    alt((
        parse_variants.map(|v: Vec<Variant>| {
            CustomTypeContent::Enum(
                v.into_iter()
                    .enumerate()
                    .map(|(i, v)| (i as u8, v))
                    .collect(),
            )
        }),
        parse_fields.map(|v: Vec<(String, Type)>| {
            let mut fields: Vec<Field> = v.into_iter().map(|(s, t)| Field::new(&s, t)).collect();
            fields.sort_by_key(|f| f.name().to_owned());
            CustomTypeContent::Record(fields)
        }),
    ))
    .parse_next(input)
}

fn parse_variants(input: &mut &str) -> Result<Vec<Variant>> {
    separated(1.., parse_variant, ws(',')).parse_next(input)
}

fn parse_fields(input: &mut &str) -> Result<Vec<(String, Type)>> {
    separated(1.., parse_field, ws(',')).parse_next(input)
}

fn parse_variant(input: &mut &str) -> Result<Variant> {
    let name = parse_identifier_upper(input)?;
    opt(delimited('(', separated(0.., parse_type, ws(',')), ')'))
        .map(|types| Variant::new(name, types.unwrap_or_default()))
        .parse_next(input)
}

fn parse_field(input: &mut &str) -> Result<(String, Type)> {
    let name = parse_identifier_lower(input)?;
    let _ = ws(':').parse_next(input)?;
    let ty = parse_type(input)?;
    Ok((name.to_owned(), ty))
}

#[cfg(test)]
mod tests {
    use ast::types::{
        Type,
        custom::{CustomTypeContent, Field, Variant},
    };

    #[test]
    fn test_parse_empty_custom_type() {
        let mut input = "type Phantom";
        let parsed = super::parse_custom_type_definition(&mut input).unwrap();
        assert_eq!(parsed.name(), "Phantom");
        assert_eq!(parsed.content(), None);
    }

    #[test]
    fn test_parse_custom_type_variants() {
        let mut input = "type CustomType { VariantA, VariantB, }";
        let parsed = super::parse_custom_type_definition(&mut input).unwrap();
        assert_eq!(parsed.name(), "CustomType");
        assert_eq!(
            parsed.content(),
            Some(&CustomTypeContent::Enum(vec![
                (0, Variant::new("VariantA", vec![])),
                (1, Variant::new("VariantB", vec![])),
            ]))
        );
    }

    #[test]
    fn test_parse_custom_type_generics() {
        let mut input = "type Option<A> { Some(A), None }";
        let parsed = super::parse_custom_type_definition(&mut input).unwrap();
        assert_eq!(parsed.name(), "Option");
        assert_eq!(
            parsed.content(),
            Some(&CustomTypeContent::Enum(vec![
                (
                    0,
                    Variant::new("Some", vec![Type::Custom("A".to_owned(), vec![])])
                ),
                (1, Variant::new("None", vec![])),
            ]))
        );
    }

    #[test]
    fn test_parse_custom_type_record() {
        let mut input = "type CustomType { admin: Bool, age: U8, }";
        let parsed = super::parse_custom_type_definition(&mut input).unwrap();
        assert_eq!(parsed.name(), "CustomType");
        assert_eq!(
            parsed.content(),
            Some(&CustomTypeContent::Record(vec![
                Field::new("admin", Type::Bool),
                Field::new("age", Type::U8),
            ]))
        );
    }
}
