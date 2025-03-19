// Perform type inference on the AST. Usually by replacing `Type::Pending` with a concrete type.

use core::panic;
use std::collections::BTreeMap;

use crate::frontend::ast::{
    expressions::{Expression, ExpressionKind, MatchArm},
    program::Program,
    types::{custom::CustomTypeContent, CustomType, Type},
};

#[derive(Default)]
pub struct TypeInference {
    variables: BTreeMap<String, Type>,
    types: Vec<CustomType>,
    functions: BTreeMap<String, Type>,
}

impl TypeInference {
    pub fn infer_program(&mut self, program: &mut Program) {
        for fd in &program.function_declarations {
            self.functions
                .insert(fd.name().to_owned(), fd.signature().return_type().clone());
        }

        for a in &program.type_definitions {
            self.types.push(a.clone());
        }

        for stmt in &mut program.entry_point.statements {
            match stmt {
                crate::frontend::ast::statements::Statement::Assignment(var_name, expression) => {
                    expression.associated_type = Some(self.infer_expr(expression));
                    let ty = expression
                        .associated_type()
                        .expect("Type should be inferred")
                        .clone();
                    self.variables.insert(var_name.to_owned(), ty);
                }
                crate::frontend::ast::statements::Statement::Return(_) => todo!(),
            }
        }

        self.infer_expr(&mut program.entry_point.return_expr);
    }

    pub fn infer_expr(&self, expression: &mut Expression) -> Type {
        expression.associated_type.clone().unwrap_or_else(|| {
            let ty = match expression.kind.clone() {
                ExpressionKind::Identifier(var_name) => {
                    self.variables.get(&var_name).unwrap().clone()
                }
                ExpressionKind::Literal(literal) => literal.get_type(),
                ExpressionKind::BinaryOp(mut lhs, _, mut rhs) => {
                    self.infer_binary_op(&mut lhs, &mut rhs)
                }
                ExpressionKind::UnaryOp(_, mut expression) => self.infer_expr(&mut expression),
                ExpressionKind::Unit => Type::Unit,
                ExpressionKind::FunctionCall(name, _) => self.functions.get(&name).unwrap().clone(),
                ExpressionKind::Match(mut expr, mut arms) => self.infer_match(&mut expr, &mut arms),
                ExpressionKind::RecordAccess(var_name, field_name) => {
                    let ty = self.variables.get(&var_name).unwrap();
                    let Type::Custom(name, _) = ty else {
                        panic!("Expected custom type for record access");
                    };
                    let ty = self.types.iter().find(|t| t.name() == name).unwrap();
                    eprintln!("Found type: {:?}", ty);
                    let CustomTypeContent::Record(a) = ty.content() else {
                        panic!("Expected record type for record access");
                    };
                    a.iter()
                        .find(|(name, _)| name == &field_name)
                        .map(|(_, value)| value.clone())
                        .expect("Record field not found")
                }
                _ => todo!(),
            };
            ty
        })
    }

    fn infer_binary_op(&self, lhs: &mut Expression, rhs: &mut Expression) -> Type {
        let lhs_type = self.infer_expr(lhs);
        let rhs_type = self.infer_expr(rhs);
        if lhs_type != rhs_type {
            panic!()
        }
        lhs_type
    }

    fn infer_match(&self, expr: &mut Expression, arms: &mut [MatchArm]) -> Type {
        self.infer_expr(expr);
        let arms: Vec<_> = arms
            .iter()
            .filter_map(|arm| match &arm.body {
                crate::frontend::ast::expressions::MatchBody::Block(block) => {
                    block.return_expr.associated_type.clone()
                }
                crate::frontend::ast::expressions::MatchBody::Expr(expression) => {
                    expression.associated_type.clone()
                }
            })
            .collect();
        if arms.iter().all(|x| x == &arms[0]) {
            arms[0].clone()
        } else {
            panic!()
        }
    }
}
