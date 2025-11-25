use winnow::{
    Parser, Result,
    combinator::opt,
    stream::AsChar,
    token::{one_of, take_while},
};

// can begin with a lowercase letter or underscore, then it can be followed by any number of letters, numbers, or underscores
pub fn parse_identifier_lower<'s>(input: &mut &'s str) -> Result<&'s str> {
    (
        one_of(|c: char| c.is_alpha() && c.is_lowercase() || c == '_'),
        take_while(0.., |c: char| c.is_alphanum() || c == '_'),
    )
        .take()
        .parse_next(input)
}

// can begin with an uppercase letter or underscore followed by an uppercase letter, then it can be followed by any number of letters or numbers
pub fn parse_identifier_upper<'s>(input: &mut &'s str) -> Result<&'s str> {
    (
        opt('_'),
        one_of(|c: char| c.is_alpha() && c.is_uppercase()),
        take_while(0.., |c: char| c.is_alphanum() || c == '_').verify(|c: &str| !c.contains('_')),
    )
        .take()
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identifier_lower() {
        let mut input = "my_value12";
        let parsed = parse_identifier_lower(&mut input).unwrap();
        assert_eq!(parsed, "my_value12");
    }

    #[test]
    fn test_parse_identifier_lower_underscore() {
        let mut input = "_my_value";
        let parsed = parse_identifier_lower(&mut input).unwrap();
        assert_eq!(parsed, "_my_value");
    }

    #[test]
    fn test_parse_identifier_lower_with_type() {
        let mut input = "my_value: U8";
        let parsed = parse_identifier_lower(&mut input).unwrap();
        assert_eq!(input, ": U8");
        assert_eq!(parsed, "my_value");
    }

    #[test]
    fn test_parse_invalid_identifier_lower_is_upper() {
        let mut input = "MyType";
        let result = parse_identifier_lower(&mut input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_identifier_lower_starts_number() {
        let mut input = "12ident";
        let result = parse_identifier_lower(&mut input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_identifier_upper() {
        let mut input = "MyType";
        let parsed = parse_identifier_upper(&mut input).unwrap();
        assert_eq!(parsed, "MyType");
    }

    #[test]
    fn test_parse_identifier_upper_underscore() {
        let mut input = "_MyType";
        let parsed = parse_identifier_upper(&mut input).unwrap();
        assert_eq!(parsed, "_MyType");
    }

    #[test]
    fn test_parse_invalid_identifier_upper_is_lower() {
        let mut input = "my_value";
        let result = parse_identifier_upper(&mut input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_identifier_upper_starts_lower() {
        let mut input = "_myValue";
        let result = parse_identifier_upper(&mut input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_identifier_upper_has_middle_underscore() {
        let mut input = "My_Value";
        let result = parse_identifier_upper(&mut input);
        assert!(result.is_err());
    }
}
