use std::collections::BTreeMap;

use ast::{
    expressions::{AssociatedType, Expression, ExpressionKind, Literal, MatchArm, MatchBody},
    program::Program,
    statements::Statement,
    types::{CustomType, FunctionSignature, Type},
};

#[derive(Default)]
pub struct TypeInference {
    variables: BTreeMap<String, Type>,
    types: Vec<CustomType>,
    functions: BTreeMap<String, FunctionSignature>,
    constraints: Vec<Constraint>,
}

struct Constraint;

impl TypeInference {
    pub fn infer_program(&mut self, program: &mut Program) {
        for fd in &program.function_declarations {
            self.functions
                .insert(fd.name().to_owned(), fd.signature().to_owned());
        }

        for a in &program.type_definitions {
            self.types.push(a.to_owned());
        }

        for stmt in &mut program.entry_point.statements {
            match stmt {
                Statement::Assignment(var_name, expr) => {
                    let new_ty = self.infer_expr(expr);
                    expr.set_type(AssociatedType::Concrete(new_ty.to_owned()));
                    self.variables.insert(var_name.to_owned(), new_ty);
                }
                Statement::Return(_) => todo!(),
            }
        }

        self.infer_expr(&mut program.entry_point.return_expr);
    }

    pub fn infer_expr(&self, expr: &mut Expression) -> Type {
        let ty = match expr.kind_mut() {
            ExpressionKind::Identifier(var_name) => {
                self.variables.get(var_name).unwrap().to_owned()
            }
            ExpressionKind::Literal(lit) => match lit {
                Literal::Integer(_) => Type::U8,
                Literal::Float(_) => Type::F64,
                Literal::Bool(_) => Type::Bool,
                Literal::String(_) => Type::String,
            },
            ExpressionKind::BinaryOp(lhs, _, rhs) => self.infer_binary_op(lhs, rhs),
            ExpressionKind::UnaryOp(_, expr) => self.infer_expr(expr),
            ExpressionKind::Unit => Type::Unit,
            ExpressionKind::FunctionCall(name, _) => self
                .functions
                .get(name)
                .map(|s| s.return_type().to_owned())
                .unwrap(),
            ExpressionKind::Match(expr, arms) => self.infer_match(expr, arms),
            ExpressionKind::NewRecordInstance(type_name, fields) => {
                self.infer_new_record_instance(type_name, fields)
            }
            ExpressionKind::RecordAccess(var_name, field_name) => {
                self.infer_record_access(var_name, field_name)
            }
            _ => todo!("Expression {expr:?} is not implemented yet"),
        };
        expr.set_type(AssociatedType::Concrete(ty.to_owned()));
        ty
    }

    fn infer_binary_op(&self, lhs: &mut Expression, rhs: &mut Expression) -> Type {
        let lhs_type = self.infer_expr(lhs);
        let rhs_type = self.infer_expr(rhs);
        if lhs_type != rhs_type {
            panic!("type mismatch")
        }
        lhs_type
    }

    fn infer_match(&self, expr: &mut Expression, arms: &[MatchArm]) -> Type {
        self.infer_expr(expr);
        let arms: Vec<_> = arms
            .iter()
            .filter_map(|arm| match &arm.body {
                MatchBody::Block(block) => Some(block.return_expr.associated_type.to_owned()),
                MatchBody::Expr(expr) => Some(expr.associated_type.to_owned()),
            })
            .collect();
        if arms.iter().all(|x| x == &arms[0]) {
            arms[0]
                .to_owned()
                .and_then(|t| match t {
                    AssociatedType::Concrete(ty) => Some(ty),
                    AssociatedType::Unknown(_) => None,
                })
                .unwrap()
        } else {
            panic!("type mismatch in match arms")
        }
    }

    fn infer_new_record_instance(
        &self,
        type_name: &str,
        fields: &mut [(String, Expression)],
    ) -> Type {
        let ty = self
            .types
            .iter()
            .find(|t| t.name() == type_name)
            .expect("Type not found");
        let original_fields = ty.get_record_fields().expect("Type is not a record");
        original_fields.iter().for_each(|f| {
            let (_, expr) = fields
                .iter_mut()
                .find(|(field_name, _)| f.name() == field_name)
                .expect("Field not found");
            expr.set_type(AssociatedType::Concrete(f.ty().to_owned()));
        });
        Type::Custom(type_name.to_owned(), vec![])
    }

    fn infer_record_access(&self, var_name: &str, field_name: &str) -> Type {
        let ty = self.variables.get(var_name).expect("Variable not found");
        let Type::Custom(name, _) = ty else {
            panic!("Expected custom type for record access");
        };
        self.types
            .iter()
            .find(|t| t.name() == name)
            .and_then(|t| t.get_record_fields())
            .expect("Record type not found")
            .iter()
            .find(|f| f.name() == field_name)
            .map(|f| f.ty().to_owned())
            .expect("Record field not found")
    }
}
