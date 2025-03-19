use core::panic;

use cranelift::prelude::{types, FunctionBuilder, InstBuilder, IntCC, Value};
use cranelift_module::Module;

use crate::frontend::ast::expressions::{BinaryOp, Expression, Literal};

use super::Codegen;

impl Codegen {
    pub fn gen_expression(
        &mut self,
        expression: &Expression,
        builder: &mut FunctionBuilder,
    ) -> Value {
        use crate::frontend::ast::expressions::ExpressionKind;

        match &expression.kind {
            ExpressionKind::Literal(literal) => gen_literal(literal, builder),
            ExpressionKind::Identifier(var_name) => {
                let (var, _) = self.get_variable(var_name).unwrap();
                builder.use_var(*var)
            }
            ExpressionKind::BinaryOp(lhs, op, rhs) => self.gen_binary_op(lhs, rhs, op, builder),
            ExpressionKind::FunctionCall(function_name, args) => {
                self.gen_function_call(function_name, args, builder)
            }
            ExpressionKind::Unit => builder.ins().iconst(types::I32, 0),
            ExpressionKind::NewRecordInstance(a, b) => self.gen_new_record_instance(a, b, builder),
            ExpressionKind::RecordAccess(a, b) => self.gen_record_access(a, b, builder),
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

    fn gen_function_call(
        &mut self,
        function_name: &str,
        args: &[Expression],
        builder: &mut FunctionBuilder,
    ) -> Value {
        let args: Vec<_> = args
            .iter()
            .map(|e| self.gen_expression(e, builder))
            .collect();
        let (func_id, _) = self.get_function(function_name).unwrap();
        let fref = self.module.declare_func_in_func(*func_id, builder.func);

        let i = builder.ins().call(fref, &args);
        let val = builder.inst_results(i)[0];
        val
    }

    fn gen_new_record_instance(
        &mut self,
        record_name: &str,
        fields: &[(String, Expression)],
        builder: &mut FunctionBuilder,
    ) -> Value {
        todo!()
    }

    fn gen_record_access(
        &mut self,
        record_name: &str,
        field_name: &str,
        builder: &mut FunctionBuilder,
    ) -> Value {
        todo!()
    }
}

fn gen_literal(lit: &Literal, builder: &mut FunctionBuilder) -> Value {
    match lit {
        Literal::Integer(v, ty) => builder.ins().iconst(ty.to_cranelift(), *v as i64),
        Literal::Float(v, ty) => match ty {
            crate::frontend::ast::types::Type::F32 => builder.ins().f32const(*v as f32),
            crate::frontend::ast::types::Type::F64 => builder.ins().f64const(*v),
            _ => panic!("this is never supposed to happen!"),
        },
        Literal::Bool(v) => builder.ins().iconst(types::I8, if *v { 1 } else { 0 }),
        Literal::String(_) => todo!(),
    }
}
