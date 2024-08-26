use nom::character::complete::{alpha1, alphanumeric1, multispace0};
use nom::combinator::{recognize, verify};
use nom::error::ParseError;
use nom::multi::many0_count;
use nom::sequence::{delimited, pair, preceded};
use nom::{branch::alt, bytes::complete::tag};
use nom::{IResult, Parser};

pub fn ws<'a, O, E: ParseError<&'a str>, F: Parser<&'a str, O, E>>(
    f: F,
) -> impl Parser<&'a str, O, E> {
    delimited(multispace0, f, multispace0)
}

// can begin with a lowercase letter or underscore, then it can be followed by any number of letters, numbers, or underscores
pub fn parse_identifier_lower(input: &str) -> IResult<&str, &str> {
    preceded(
        multispace0,
        recognize(pair(
            alt((
                verify(alpha1, |s: &str| {
                    s.chars().next().map(|c| c.is_lowercase()).unwrap_or(false)
                }),
                tag("_"),
            )),
            many0_count(alt((alphanumeric1, tag("_")))),
        )),
    )
    .parse(input)
}

// can begin with an uppercase letter or underscore, then it can be followed by any number of letters, numbers, or underscores
pub fn parse_identifier_upper(input: &str) -> IResult<&str, &str> {
    ws(recognize(pair(
        alt((
            verify(alpha1, |s: &str| {
                s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
            }),
            tag("_"),
        )),
        many0_count(alt((alphanumeric1, tag("_")))),
    )))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identifier_lower() {
        let input = "my_value";
        let (_, parsed) = parse_identifier_lower(input).unwrap();
        assert_eq!(parsed, input);
    }

    #[test]
    fn test_parse_identifier_lower_underscore() {
        let input = "_my_value";
        let (_, parsed) = parse_identifier_lower(input).unwrap();
        assert_eq!(parsed, input);
    }

    #[test]
    fn test_parse_identifier_lower_with_type() {
        let input = "my_value: U8";
        let (remaining, parsed) = parse_identifier_lower(input).unwrap();
        assert_eq!(remaining, ": U8");
        assert_eq!(parsed, "my_value");
    }

    #[test]
    fn test_parse_identifier_lower_spaces() {
        let input = "  my_value ";
        let (_, parsed) = parse_identifier_lower(input).unwrap();
        assert_eq!(parsed, "my_value");
    }

    #[test]
    fn test_parse_invalid_identifier_lower() {
        let input = "MyType";
        let result = parse_identifier_lower(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_identifier_upper() {
        let input = "MyType";
        let (_, parsed) = parse_identifier_upper(input).unwrap();
        assert_eq!(parsed, input);
    }

    #[test]
    fn test_parse_identifier_upper_underscore() {
        let input = "_MyType";
        let (_, parsed) = parse_identifier_upper(input).unwrap();
        assert_eq!(parsed, input);
    }

    #[test]
    fn test_parse_identifier_upper_spaces() {
        let input = "  MyType ";
        let (_, parsed) = parse_identifier_upper(input).unwrap();
        assert_eq!(parsed, "MyType");
    }

    #[test]
    fn test_parse_invalid_identifier_upper() {
        let input = "my_value";
        let result = parse_identifier_upper(input);
        assert!(result.is_err());
    }
}
