use super::expressions::Literal;

/// A pattern, as used in match arms and function implementation parameters.
///
/// Patterns are NOT expressions: they are a restricted, destructuring-only
/// subset. (`1 + 2 -> x` is not a valid pattern.)
///
/// Grammar reference (`grammar_optimized.pest`): `Pattern`
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// `_` — matches anything, binds nothing
    Wildcard,
    /// `x` — matches anything, binds it to a name
    Identifier(String),
    /// `0`, `True`, `"hello"`
    Literal(Literal),
    /// `Option::Some(x)`, `Status::Ready`
    EnumInstance {
        enum_name: String,
        variant_name: String,
        args: Vec<Self>,
    },
    // TODO: StructPat (`Client { name, age }`) and TuplePat (`(a, b)`) once
    // records/tuples are settled on the expression side.
}
