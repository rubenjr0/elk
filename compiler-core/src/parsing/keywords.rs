use nom::{branch::alt, bytes::complete::tag, combinator::value, IResult, Parser};

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Type,
    Main,
    If,
    Else,
    Match,
}

pub fn parse_keyword(input: &str) -> IResult<&str, Keyword> {
    alt((
        value(Keyword::Type, tag("type")),
        value(Keyword::Main, tag("main")),
        value(Keyword::If, tag("if")),
        value(Keyword::Else, tag("else")),
        value(Keyword::Match, tag("match")),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_keyword_type() {
        let input = "type";
        let (_, parsed) = parse_keyword(input).unwrap();
        assert_eq!(parsed, Keyword::Type);
    }

    #[test]
    fn test_parse_keyword_main() {
        let input = "main";
        let (_, parsed) = parse_keyword(input).unwrap();
        assert_eq!(parsed, Keyword::Main);
    }

    #[test]
    fn test_parse_keyword_match() {
        let input = "match";
        let (_, parsed) = parse_keyword(input).unwrap();
        assert_eq!(parsed, Keyword::Match);
    }

    #[test]
    fn test_parse_keyword_if() {
        let input = "if";
        let (_, parsed) = parse_keyword(input).unwrap();
        assert_eq!(parsed, Keyword::If);
    }

    #[test]
    fn test_parse_keyword_else() {
        let input = "else";
        let (_, parsed) = parse_keyword(input).unwrap();
        assert_eq!(parsed, Keyword::Else);
    }
}
