use cranelift::prelude::{types, FunctionBuilder, InstBuilder, Value};
use cranelift_module::Module;

use crate::frontend::ast::expressions::{Expression, Literal};

use super::Codegen;

impl Codegen {
    pub fn gen_expression(
        &mut self,
        expression: &Expression,
        builder: &mut FunctionBuilder,
    ) -> Value {
        use crate::frontend::ast::expressions::{BinaryOp, ExpressionKind};

        match &expression.kind {
            ExpressionKind::Literal(literal) => gen_literal(literal, builder),
            ExpressionKind::Identifier(var_name) => {
                let (_, var) = self.variables.get(var_name).unwrap();
                builder.use_var(*var)
            }
            ExpressionKind::BinaryOp(lhs, op, rhs) => {
                let lhs = self.gen_expression(lhs, builder);
                let rhs = self.gen_expression(rhs, builder);
                match op {
                    BinaryOp::Add => builder.ins().iadd(lhs, rhs),
                    BinaryOp::Sub => builder.ins().isub(lhs, rhs),
                    BinaryOp::Mul => builder.ins().imul(lhs, rhs),
                    BinaryOp::And => builder.ins().iadd(lhs, rhs),
                    BinaryOp::Or => builder.ins().bor(lhs, rhs),
                    _ => todo!(),
                }
            }
            ExpressionKind::FunctionCall(function_name, args) => {
                let args: Vec<_> = args
                    .iter()
                    .map(|e| self.gen_expression(e, builder))
                    .collect();
                let (func_id, _, _) = self.functions.get(function_name).unwrap();
                let fref = self
                    .module
                    .declare_func_in_func(*func_id, &mut builder.func);

                let i = builder.ins().call(fref, &args);
                let val = Value::from_bits(i.as_u32());
                val
            }
            ExpressionKind::Unit => builder.ins().iconst(types::I32, 0),
            _ => todo!(),
        }
    }
}

fn gen_literal(lit: &Literal, builder: &mut FunctionBuilder) -> Value {
    match lit {
        Literal::Integer(v, ty) => builder.ins().iconst(ty.into(), *v as i64),
        Literal::Float(v, ty) => builder.ins().iconst(ty.into(), *v as i64),
        Literal::Bool(v) => builder.ins().iconst(types::I8, if *v { 1 } else { 0 }),
        Literal::String(_) => todo!(),
    }
}
