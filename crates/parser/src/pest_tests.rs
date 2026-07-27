/// Pest grammar tests — the grammar is the source of truth.
/// These mirror the winnow tests but validate the pest grammar directly,
/// without going through the AST. Once these reach parity, winnow can be sunset.
#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::{Grammar, Rule};

    // ==========================================
    // Helpers
    // ==========================================

    /// Asserts that `rule` fully consumes `input`.
    /// For non-silent rules, checks the span covers the entire input.
    fn assert_parses(rule: Rule, input: &str) {
        match Grammar::parse(rule, input) {
            Err(e) => panic!("Expected {rule:?} to parse {input:?}\n{e}"),
            Ok(mut pairs) => {
                let consumed = pairs.next().map(|p| p.as_span().end()).unwrap_or(0);
                assert_eq!(
                    consumed,
                    input.len(),
                    "{rule:?} consumed {consumed}/{} chars in {input:?}",
                    input.len()
                );
            }
        }
    }

    /// Asserts that `rule` does NOT fully consume `input`
    /// (either parsing fails outright or only a prefix is matched).
    fn assert_fails(rule: Rule, input: &str) {
        let fully_parsed = Grammar::parse(rule, input)
            .ok()
            .and_then(|mut p| p.next())
            .map(|p| p.as_span().end() == input.len())
            .unwrap_or(false);
        assert!(!fully_parsed, "Expected {rule:?} NOT to fully parse {input:?}");
    }

    /// Asserts a program parses fully. `Program` is a silent rule with SOI+EOI,
    /// so a successful parse already guarantees full consumption.
    fn assert_program_parses(input: &str) {
        Grammar::parse(Rule::Program, input)
            .unwrap_or_else(|e| panic!("Failed to parse program:\n{e}"));
    }

    // ==========================================
    // Literals
    // ==========================================

    #[test]
    fn test_literal_bool_true() {
        assert_parses(Rule::Boolean, "True");
    }

    #[test]
    fn test_literal_bool_false() {
        assert_parses(Rule::Boolean, "False");
    }

    #[test]
    fn test_literal_integer() {
        assert_parses(Rule::Integer, "37");
    }

    #[test]
    fn test_literal_integer_negative() {
        // Literals are non-negative; `-37` is UnaryOp + Literal at the expression level.
        assert_parses(Rule::Expr, "-37");
    }

    #[test]
    fn test_literal_binary() {
        assert_parses(Rule::BinInt, "0b110");
    }

    #[test]
    fn test_literal_binary_uppercase() {
        assert_parses(Rule::BinInt, "0B1010");
    }

    #[test]
    fn test_literal_octal() {
        assert_parses(Rule::OctInt, "0o20");
    }

    #[test]
    fn test_literal_octal_uppercase() {
        assert_parses(Rule::OctInt, "0O77");
    }

    #[test]
    fn test_literal_hexadecimal() {
        assert_parses(Rule::HexInt, "0x20");
    }

    #[test]
    fn test_literal_hexadecimal_uppercase_prefix() {
        assert_parses(Rule::HexInt, "0XFF");
    }

    #[test]
    fn test_literal_float() {
        assert_parses(Rule::Float, "0.12");
    }

    #[test]
    fn test_literal_float_negative() {
        // Literals are non-negative; `-1.5` is UnaryOp + Literal at the expression level.
        assert_parses(Rule::Expr, "-1.5");
    }

    #[test]
    fn test_literal_string_empty() {
        assert_parses(Rule::StringLit, r#""""#);
    }

    #[test]
    fn test_literal_string_simple() {
        assert_parses(Rule::StringLit, r#""hello world""#);
    }

    #[test]
    fn test_literal_string_escaped_quote() {
        // Input: "hello, \"world\""
        assert_parses(Rule::StringLit, r#""hello, \"world\"""#);
    }

    #[test]
    fn test_literal_string_escape_sequences() {
        assert_parses(Rule::StringLit, r#""line1\nline2\ttabbed""#);
    }

    #[test]
    fn test_literal_unit() {
        assert_parses(Rule::UnitLit, "()");
    }

    // Number dispatches to the correct sub-rule
    #[test]
    fn test_number_integer() {
        assert_parses(Rule::Number, "42");
    }

    #[test]
    fn test_number_float_preferred_over_integer() {
        // "3.14" must parse as Float, not Integer "3" + leftover ".14"
        assert_parses(Rule::Number, "3.14");
    }

    #[test]
    fn test_number_hex() {
        assert_parses(Rule::Number, "0xFF");
    }

    #[test]
    fn test_number_binary() {
        assert_parses(Rule::Number, "0b1010");
    }

    #[test]
    fn test_number_octal() {
        assert_parses(Rule::Number, "0o77");
    }

    // ==========================================
    // Identifiers
    // ==========================================

    #[test]
    fn test_identifier_lower() {
        assert_parses(Rule::Identifier, "my_value12");
    }

    #[test]
    fn test_identifier_lower_leading_underscore() {
        assert_parses(Rule::Identifier, "_my_value");
    }

    #[test]
    fn test_identifier_upper() {
        assert_parses(Rule::TypeIdentifier, "MyType");
    }

    #[test]
    fn test_identifier_upper_leading_underscore() {
        assert_parses(Rule::TypeIdentifier, "_MyType");
    }

    // Grammar allows underscores mid-TypeIdentifier — differs from the winnow parser.
    #[test]
    fn test_identifier_upper_mid_underscore_allowed() {
        assert_parses(Rule::TypeIdentifier, "My_Value");
    }

    #[test]
    fn test_identifier_lower_rejects_uppercase_start() {
        assert_fails(Rule::Identifier, "MyType");
    }

    #[test]
    fn test_identifier_lower_rejects_digit_start() {
        assert_fails(Rule::Identifier, "12ident");
    }

    #[test]
    fn test_identifier_upper_rejects_lowercase_start() {
        assert_fails(Rule::TypeIdentifier, "my_value");
    }

    #[test]
    fn test_identifier_rejects_keyword_main() {
        assert_fails(Rule::Identifier, "main");
    }

    #[test]
    fn test_identifier_rejects_keyword_type() {
        assert_fails(Rule::Identifier, "type");
    }

    #[test]
    fn test_identifier_rejects_keyword_match() {
        assert_fails(Rule::Identifier, "match");
    }

    // Keyword prefix followed by more chars is a valid identifier
    #[test]
    fn test_identifier_allows_keyword_prefix() {
        assert_parses(Rule::Identifier, "main_fn");
        assert_parses(Rule::Identifier, "matcher");
        assert_parses(Rule::Identifier, "type_alias");
    }

    // ==========================================
    // Types
    // ==========================================

    #[test]
    fn test_type_primitive_u8() {
        assert_parses(Rule::TypeRef, "U8");
    }

    #[test]
    fn test_type_primitive_i64() {
        assert_parses(Rule::TypeRef, "I64");
    }

    #[test]
    fn test_type_primitive_bool() {
        assert_parses(Rule::TypeRef, "Bool");
    }

    #[test]
    fn test_type_unit() {
        assert_parses(Rule::UnitType, "()");
    }

    #[test]
    fn test_type_custom() {
        assert_parses(Rule::TypeRef, "MyType");
    }

    #[test]
    fn test_type_custom_generic() {
        assert_parses(Rule::TypeRef, "Option<T>");
    }

    #[test]
    fn test_type_custom_multi_generic() {
        assert_parses(Rule::TypeRef, "Result<T, E>");
    }

    #[test]
    fn test_type_list() {
        assert_parses(Rule::ListType, "[U8]");
    }

    #[test]
    fn test_type_list_nested() {
        assert_parses(Rule::ListType, "[[U8]]");
    }

    #[test]
    fn test_type_tuple() {
        assert_parses(Rule::TupleType, "(U8, String)");
    }

    #[test]
    fn test_type_tuple_three() {
        assert_parses(Rule::TupleType, "(U8, String, Bool)");
    }

    #[test]
    fn test_type_function_simple() {
        assert_parses(Rule::FunctionType, "(U8) -> Bool");
    }

    #[test]
    fn test_type_function_no_args() {
        assert_parses(Rule::FunctionType, "() -> ()");
    }

    #[test]
    fn test_type_function_multi_arg() {
        assert_parses(Rule::FunctionType, "(U8, String) -> Bool");
    }

    #[test]
    fn test_type_function_labeled_args() {
        assert_parses(Rule::FunctionType, "(width: U8, height: U8) -> U8");
    }

    #[test]
    fn test_type_function_higher_order() {
        // Matches: `(f: ((U8) -> U8), n: U8) -> U8`
        assert_parses(Rule::FunctionType, "(f: (U8) -> U8, n: U8) -> U8");
    }

    // ==========================================
    // Custom Type Definitions
    // ==========================================

    #[test]
    fn test_custom_type_marker() {
        assert_parses(Rule::CustomTypeDef, "type Inactive;");
    }

    #[test]
    fn test_custom_type_marker_generic() {
        assert_parses(Rule::CustomTypeDef, "type StateMachine<S>;");
    }

    #[test]
    fn test_custom_type_enum_simple() {
        assert_parses(Rule::CustomTypeDef, "type Status { Loading, Ready, Error }");
    }

    #[test]
    fn test_custom_type_enum_with_data() {
        assert_parses(Rule::CustomTypeDef, "type Option<T> { None, Some(T) }");
    }

    #[test]
    fn test_custom_type_enum_trailing_comma() {
        assert_parses(Rule::CustomTypeDef, "type Status { Loading, Ready, }");
    }

    #[test]
    fn test_custom_type_struct() {
        assert_parses(Rule::CustomTypeDef, "type Client { name: String, age: U8 }");
    }

    #[test]
    fn test_custom_type_struct_trailing_comma() {
        assert_parses(Rule::CustomTypeDef, "type Client { name: String, age: U8, }");
    }

    // ==========================================
    // Expressions
    // ==========================================

    #[test]
    fn test_expr_identifier() {
        assert_parses(Rule::Expr, "my_var");
    }

    #[test]
    fn test_expr_unit() {
        assert_parses(Rule::Expr, "()");
    }

    #[test]
    fn test_expr_binary_add() {
        assert_parses(Rule::Expr, "a + b");
    }

    #[test]
    fn test_expr_binary_complex() {
        assert_parses(Rule::Expr, "a + b * c");
    }

    #[test]
    fn test_expr_unary_not() {
        assert_parses(Rule::Expr, "!True");
    }

    #[test]
    fn test_expr_unary_neg() {
        assert_parses(Rule::Expr, "-x");
    }

    #[test]
    fn test_expr_field_access() {
        assert_parses(Rule::Postfix, "person.name");
    }

    #[test]
    fn test_expr_field_access_chained() {
        // Grammar supports arbitrary depth; winnow only supported one level.
        assert_parses(Rule::Postfix, "a.b.c");
    }

    #[test]
    fn test_expr_enum_instance_no_args() {
        assert_parses(Rule::NewEnumInstance, "Option.None");
    }

    #[test]
    fn test_expr_enum_instance_with_args() {
        assert_parses(Rule::NewEnumInstance, "Option.Some(1)");
    }

    #[test]
    fn test_expr_struct_instance() {
        assert_parses(Rule::NewStructInstance, r#"Person { name: "Bob", is_builder: True }"#);
    }

    #[test]
    fn test_expr_function_call() {
        assert_parses(Rule::FunctionCall, "my_function(arg1, arg2)");
    }

    #[test]
    fn test_expr_function_call_nested() {
        assert_parses(Rule::FunctionCall, r#"my_function(other_fn(42), Person { name: "Bob" })"#);
    }

    #[test]
    fn test_expr_function_call_namespaced() {
        assert_parses(Rule::FunctionCall, "Option::map(opt, f)");
    }

    #[test]
    fn test_expr_match_bool() {
        // Grammar uses `=>` for match arms (winnow tests incorrectly used `->`)
        assert_parses(Rule::MatchBlock, "match my_bool { True => 1, False => 0 }");
    }

    #[test]
    fn test_expr_match_enum_patterns() {
        assert_parses(
            Rule::MatchBlock,
            "match opt { Option.Some(x) => x, Option.None => 0 }",
        );
    }

    #[test]
    fn test_expr_match_wildcard() {
        assert_parses(Rule::MatchBlock, "match x { 0 => True, _ => False }");
    }

    #[test]
    fn test_expr_list_literal() {
        assert_parses(Rule::ListLit, "[1, 2, 3]");
    }

    #[test]
    fn test_expr_list_literal_empty() {
        assert_parses(Rule::ListLit, "[]");
    }

    #[test]
    fn test_expr_tuple_literal() {
        assert_parses(Rule::TupleLit, "(1, True)");
    }

    #[test]
    fn test_expr_lambda_single_arg() {
        assert_parses(Rule::Lambda, "(x) -> x");
    }

    #[test]
    fn test_expr_lambda_multi_arg() {
        assert_parses(Rule::Lambda, "(x, y) -> x");
    }

    #[test]
    fn test_expr_lambda_no_args() {
        assert_parses(Rule::Lambda, "() -> 42");
    }

    #[test]
    fn test_expr_lambda_with_body() {
        assert_parses(Rule::Lambda, "(x) -> x + 1");
    }

    #[test]
    fn test_expr_pipe_forward() {
        assert_parses(Rule::Pipe, "a_number |> double |> increment");
    }

    #[test]
    fn test_expr_pipe_partial_application() {
        assert_parses(Rule::Pipe, "opt |> Option::map(double)");
    }

    // ==========================================
    // Functions
    // ==========================================

    #[test]
    fn test_function_def_declaration_only() {
        assert_parses(Rule::FunctionDef, "my_function(U8, U8) -> U8;");
    }

    #[test]
    fn test_function_def_labeled_params() {
        assert_parses(Rule::FunctionDef, "sum(a: U8, b: U8) -> U8;");
    }

    #[test]
    fn test_function_def_no_params() {
        assert_parses(Rule::FunctionDef, "answer() -> U8;");
    }

    #[test]
    fn test_function_def_inline_body() {
        // Declaration + inline body in one: not supported by winnow but valid per grammar.
        assert_parses(Rule::FunctionDef, "sum(a: U8, b: U8) -> U8 = a + b;");
    }

    #[test]
    fn test_function_def_namespaced() {
        assert_parses(
            Rule::FunctionDef,
            "Option::map<A, B>(Self<A>, (A) -> B) -> Self<B>;",
        );
    }

    #[test]
    fn test_function_impl_wildcard() {
        assert_parses(Rule::FunctionImpl, "my_function(_x) = ();");
    }

    #[test]
    fn test_function_impl_multi_arg() {
        assert_parses(Rule::FunctionImpl, "my_function(_x, _y) = 1;");
    }

    #[test]
    fn test_function_impl_block_body() {
        // Grammar requires `= { ... };` — the `=` before the block is mandatory.
        assert_parses(Rule::FunctionImpl, "my_function(_x, _y) = { _z = 1; _z };");
    }

    #[test]
    fn test_wildcard_pattern_alone() {
        // `_` alone must parse as WildcardPat, not Identifier
        assert_parses(Rule::WildcardPat, "_");
    }

    #[test]
    fn test_wildcard_pattern_in_match() {
        assert_parses(Rule::MatchBlock, "match x { 1 => True, _ => False }");
    }

    #[test]
    fn test_function_impl_wildcard_pattern() {
        assert_parses(Rule::FunctionImpl, "is_origin(_) = False;");
    }

    #[test]
    fn test_function_impl_literal_pattern() {
        assert_parses(Rule::FunctionImpl, "is_zero(0) = True;");
    }

    #[test]
    fn test_function_impl_enum_pattern() {
        assert_parses(Rule::FunctionImpl, "is_some(Option.None) = False;");
    }

    #[test]
    fn test_function_impl_enum_pattern_with_binding() {
        assert_parses(Rule::FunctionImpl, "unwrap(Option.Some(x)) = x;");
    }

    // ==========================================
    // Blocks
    // ==========================================

    #[test]
    fn test_block_single_expr() {
        assert_parses(Rule::Block, "{ 42 }");
    }

    #[test]
    fn test_block_with_statements_and_expr() {
        assert_parses(Rule::Block, "{ x = 1; x }");
    }

    #[test]
    fn test_block_statements_only() {
        // No final expression — implicitly returns ()
        assert_parses(Rule::Block, "{ x = 1; y = 2; }");
    }

    // ==========================================
    // Main and Programs
    // ==========================================

    #[test]
    fn test_main_with_expr() {
        assert_parses(Rule::Main, "main { x = 1; x }");
    }

    #[test]
    fn test_main_statements_only() {
        assert_parses(Rule::Main, "main { x = 1; y = 2; }");
    }

    #[test]
    fn test_program_simple() {
        assert_program_parses(
            "sum(U8, U8) -> U8;\nsum(a, b) = a + b;\nmain { z = sum(1, 2); z }",
        );
    }

    #[test]
    fn test_program_sample() {
        assert_program_parses(include_str!("../../../samples/sample.elk"));
    }

    #[test]
    fn test_program_advanced_sample() {
        assert_program_parses(include_str!("../../../samples/advanced_sample.elk"));
    }

    #[test]
    fn test_program_simple_sample() {
        assert_program_parses(include_str!("../../../samples/simple_sample.elk"));
    }

    #[test]
    fn test_program_match_sample() {
        assert_program_parses(include_str!("../../../samples/match_sample.elk"));
    }
}
