use ast::{
    expressions::Literal,
    patterns::Pattern,
};
use winnow::{
    Parser, Result, dispatch,
    combinator::{alt, delimited, not, opt, peek, separated, terminated},
    token::{any, one_of},
};

use crate::{
    expressions::{parse_bool, parse_number, parse_string},
    identifiers::{parse_identifier_lower, parse_identifier_upper},
    ws,
};

/// Patterns are a restricted, destructuring-only subset of syntax — NOT
/// expressions. Used in match arms and function implementation parameters.
///
/// Grammar reference (`grammar_optimized.pest`): `Pattern`
pub fn parse_pattern(input: &mut &str) -> Result<Pattern> {
    dispatch! {peek(any);
        '_' => parse_wildcard_or_binding,
        '"' => parse_string.map(|s| Pattern::Literal(Literal::String(s))),
        c if c.is_ascii_digit() => parse_number.map(Pattern::Literal),
        c if c.is_ascii_uppercase() => alt((
            parse_bool.map(|b| Pattern::Literal(Literal::Bool(b))),
            parse_enum_pattern,
        )),
        _ => parse_identifier_lower.map(|s| Pattern::Identifier(s.to_owned())),
    }
    .parse_next(input)
}

/// Bare `_` is a wildcard; `_x` is an ordinary binding.
fn parse_wildcard_or_binding(input: &mut &str) -> Result<Pattern> {
    alt((
        terminated(
            '_',
            peek(not(one_of(|c: char| c.is_alphanumeric() || c == '_'))),
        )
        .map(|_| Pattern::Wildcard),
        parse_identifier_lower.map(|s| Pattern::Identifier(s.to_owned())),
    ))
    .parse_next(input)
}

/// Example: `Option::Some(x)`, `Status::Ready`
fn parse_enum_pattern(input: &mut &str) -> Result<Pattern> {
    let enum_name = parse_identifier_upper(input)?;
    let _ = "::".parse_next(input)?;
    let variant_name = parse_identifier_upper(input)?;
    let args = opt(delimited(
        '(',
        separated(0.., ws(parse_pattern), ws(',')),
        ws(')'),
    ))
    .parse_next(input)?
    .unwrap_or_default();

    Ok(Pattern::EnumInstance {
        enum_name: enum_name.to_owned(),
        variant_name: variant_name.to_owned(),
        args,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wildcard() {
        let mut input = "_";
        assert_eq!(parse_pattern(&mut input).unwrap(), Pattern::Wildcard);
    }

    #[test]
    fn test_parse_underscore_binding_is_not_wildcard() {
        let mut input = "_x";
        let parsed = parse_pattern(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(parsed, Pattern::Identifier("_x".to_owned()));
    }

    #[test]
    fn test_parse_literal_pattern() {
        let mut input = "42";
        assert_eq!(
            parse_pattern(&mut input).unwrap(),
            Pattern::Literal(Literal::int(42))
        );

        let mut input = "True";
        assert_eq!(
            parse_pattern(&mut input).unwrap(),
            Pattern::Literal(Literal::Bool(true))
        );
    }

    #[test]
    fn test_parse_enum_pattern() {
        let mut input = "Option::Some(x)";
        let parsed = parse_pattern(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            parsed,
            Pattern::EnumInstance {
                enum_name: "Option".to_owned(),
                variant_name: "Some".to_owned(),
                args: vec![Pattern::Identifier("x".to_owned())],
            }
        );
    }

    #[test]
    fn test_parse_enum_pattern_no_args() {
        let mut input = "Option::None";
        let parsed = parse_pattern(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            parsed,
            Pattern::EnumInstance {
                enum_name: "Option".to_owned(),
                variant_name: "None".to_owned(),
                args: vec![],
            }
        );
    }

    #[test]
    fn test_parse_nested_enum_pattern() {
        let mut input = "Option::Some(Option::Some(1))";
        let parsed = parse_pattern(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            parsed,
            Pattern::EnumInstance {
                enum_name: "Option".to_owned(),
                variant_name: "Some".to_owned(),
                args: vec![Pattern::EnumInstance {
                    enum_name: "Option".to_owned(),
                    variant_name: "Some".to_owned(),
                    args: vec![Pattern::Literal(Literal::int(1))],
                }],
            }
        );
    }
}
