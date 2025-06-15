use ast::statements::{Block, Statement};
use cranelift::prelude::{FunctionBuilder, Value};

use super::{Codegen, Generable};

impl Codegen {
    pub fn gen_block(&mut self, block: &Block, builder: &mut FunctionBuilder) -> Value {
        let blk = builder.create_block();
        builder.switch_to_block(blk);
        builder.seal_block(blk);
        for stmt in block.statements() {
            match stmt {
                Statement::Assignment(var_name, expression) => {
                    let ty = expression.associated_type().expect("Type not inferred");
                    let val = self.gen_expression(expression, builder);
                    let var = self.declare_variable(var_name, ty.to_owned());
                    builder.declare_var(var, ty.to_cranelift());
                    builder.def_var(var, val);
                }
                Statement::Return(expr) => {
                    return self.gen_expression(expr, builder);
                }
            }
        }
        self.gen_expression(&block.return_expr, builder)
    }
}
