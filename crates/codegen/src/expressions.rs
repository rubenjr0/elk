use core::panic;

use cranelift::prelude::{FunctionBuilder, InstBuilder, IntCC, Value, types};

use ast::{
    expressions::{BinaryOp, Expression, ExpressionKind, Literal},
    types::Type,
};

use super::{Codegen, Generable};

impl Codegen {
    pub fn gen_expression(&mut self, expr: &Expression, builder: &mut FunctionBuilder) -> Value {
        match &expr.kind {
            ExpressionKind::Literal(literal) => {
                gen_literal(literal, expr.associated_type().unwrap(), builder)
            }
            ExpressionKind::Identifier(var_name) => {
                let (var, _) = self.get_variable(var_name).unwrap();
                builder.use_var(*var)
            }
            ExpressionKind::BinaryOp(lhs, op, rhs) => self.gen_binary_op(lhs, rhs, op, builder),
            ExpressionKind::FunctionCall(function_name, args) => {
                self.gen_function_call(function_name, args, builder)
            }
            ExpressionKind::Unit => builder.ins().iconst(types::I32, 0),
            ExpressionKind::NewRecordInstance(record_name, fields) => {
                self.gen_new_record_instance(record_name, fields, builder)
            }
            ExpressionKind::RecordAccess(var_name, field_name) => {
                self.gen_record_access(var_name, field_name, builder)
            }
            ExpressionKind::NewEnumInstance(enum_name, variant_name, _) => {
                self.gen_new_enum_instance(enum_name, variant_name, builder)
            }
            _ => todo!(),
        }
    }

    fn gen_binary_op(
        &mut self,
        lhs: &Expression,
        rhs: &Expression,
        op: &BinaryOp,
        builder: &mut FunctionBuilder,
    ) -> Value {
        let lhs = self.gen_expression(lhs, builder);
        let rhs = self.gen_expression(rhs, builder);
        match op {
            BinaryOp::Add => builder.ins().iadd(lhs, rhs),
            BinaryOp::Sub => builder.ins().isub(lhs, rhs),
            BinaryOp::Mul => builder.ins().imul(lhs, rhs),
            BinaryOp::And => builder.ins().band(lhs, rhs),
            BinaryOp::Or => builder.ins().bor(lhs, rhs),
            BinaryOp::Xor => builder.ins().bxor(lhs, rhs),
            BinaryOp::Eq => builder.ins().icmp(IntCC::Equal, lhs, rhs),
            BinaryOp::NotEq => builder.ins().icmp(IntCC::NotEqual, lhs, rhs),
            _ => todo!(),
        }
    }
}

fn gen_literal(lit: &Literal, ty: &Type, builder: &mut FunctionBuilder) -> Value {
    match lit {
        Literal::Integer(v) => builder.ins().iconst(ty.to_cranelift(), *v as i64),
        Literal::Float(v) => match ty {
            Type::F32 => builder.ins().f32const(*v as f32),
            Type::F64 => builder.ins().f64const(*v),
            _ => panic!("this is never supposed to happen!"),
        },
        Literal::Bool(v) => builder.ins().iconst(types::I8, if *v { 1 } else { 0 }),
        Literal::String(_) => todo!(),
    }
}
