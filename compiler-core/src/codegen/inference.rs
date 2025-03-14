// Perform type inference on the AST. Usually by replacing `Type::Pending` with a concrete type.

use std::collections::BTreeMap;

use crate::frontend::ast::{
    expressions::{Expression, ExpressionKind},
    program::Program,
    types::Type,
};

#[derive(Default)]
pub struct TypeInference {
    variables: BTreeMap<String, Type>,
    functions: BTreeMap<String, Type>,
}

impl TypeInference {
    pub fn infer_program(&mut self, program: &mut Program) {
        for fd in &program.function_declarations {
            self.functions
                .insert(fd.name().to_owned(), fd.signature().return_type().clone());
        }

        for stmt in &mut program.entry_point.statements {
            match stmt {
                crate::frontend::ast::statements::Statement::Assignment(var_id, expression) => {
                    expression.associated_type = self.infer_type(expression);
                    self.variables
                        .insert(var_id.to_owned(), expression.associated_type.clone());
                }
                crate::frontend::ast::statements::Statement::Return(_) => todo!(),
            }
        }
    }

    pub fn infer_type(&self, expression: &mut Expression) -> Type {
        if let Type::Pending = expression.associated_type {
            expression.associated_type = match expression.kind.clone() {
                ExpressionKind::Identifier(var_name) => {
                    self.variables.get(&var_name).unwrap().clone()
                }
                ExpressionKind::Literal(literal) => literal.get_type(),
                ExpressionKind::BinaryOp(mut lhs, _, mut rhs) => {
                    self.infer_binary_op(&mut lhs, &mut rhs)
                }
                ExpressionKind::UnaryOp(_, mut expression) => self.infer_type(&mut expression),
                ExpressionKind::Unit => Type::Unit,
                ExpressionKind::FunctionCall(name, _) => self.functions.get(&name).unwrap().clone(),
                _ => todo!(),
            };
        }
        expression.associated_type.clone()
    }

    fn infer_binary_op(&self, lhs: &mut Expression, rhs: &mut Expression) -> Type {
        let lhs_type = self.infer_type(lhs);
        let rhs_type = self.infer_type(rhs);
        if lhs_type != rhs_type {
            panic!()
        }
        lhs_type
    }
}

/*
fn _infer_match_expr(match_expr: &mut Expression) {
    match &mut match_expr.kind {
        ExpressionKind::Match(_, arms) => {
            let arms_types: Vec<_> = arms.clone().iter().map(_get_match_arm_type).collect();
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
                .filter(|arm| _get_match_arm_type(arm) == Type::Pending)
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

fn _get_match_arm_type(arm: &MatchArm) -> Type {
    match &arm.body {
        MatchBody::Expr(expr) => expr.associated_type.clone(),
        MatchBody::Block(block) => block.return_expr.associated_type.clone(),
    }
}

#[cfg(test)]
mod test {
    use crate::frontend::{
        ast::types::{PrimitiveType, Type},
        inference::_infer_match_expr,
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
        _infer_match_expr(&mut expr);
        assert_eq!(expr.associated_type, Type::Primitive(PrimitiveType::U8));
    }
}
*/
