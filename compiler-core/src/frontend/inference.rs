// Perform type inference on the AST. Usually by replacing `Type::Pending` with a concrete type.

use super::ast::{
    expressions::{Expression, ExpressionKind, MatchArm, MatchBody},
    types::Type,
};

fn infer_match_expr(match_expr: &mut Expression) {
    match &mut match_expr.kind {
        ExpressionKind::Match(_, arms) => {
            let arms_types: Vec<_> = arms.clone().iter().map(get_match_arm_type).collect();
            let first_non_pending = arms_types
                .iter()
                .find(|t| t != &&Type::Pending)
                .expect("All arms are pending");
            if arms_types
                .iter()
                .filter(|t| t != &&Type::Pending)
                .any(|t| t != first_non_pending)
            {
                panic!("Arms have different types");
            }
            arms.iter_mut()
                .filter(|arm| get_match_arm_type(arm) == Type::Pending)
                .for_each(|arm| match &mut arm.body {
                    MatchBody::Expr(e) => e.associated_type = first_non_pending.clone(),
                    MatchBody::Block(b) => {
                        b.return_expr.associated_type = first_non_pending.clone()
                    }
                });
            match_expr.associated_type = first_non_pending.clone();
        }
        _ => panic!("Expected match expression"),
    }
}

fn get_match_arm_type(arm: &MatchArm) -> Type {
    match &arm.body {
        MatchBody::Expr(expr) => expr.associated_type.clone(),
        MatchBody::Block(block) => block.return_expr.associated_type.clone(),
    }
}

#[cfg(test)]
mod test {
    use crate::frontend::{
        ast::types::{PrimitiveType, Type},
        inference::infer_match_expr,
        parser,
    };

    #[test]
    fn test_infer_match_expr() {
        let (rem, mut expr) = parser::expressions::parse_expr(
            "match some_val {
                True -> 1,
                False -> 2
            }",
        )
        .unwrap();
        assert!(rem.is_empty());
        infer_match_expr(&mut expr);
        assert_eq!(expr.associated_type, Type::Primitive(PrimitiveType::U8));
    }
}
