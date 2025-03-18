use cranelift::prelude::{EntityRef, FunctionBuilder, Value, Variable};

use crate::frontend::ast::statements::Block;

use super::Codegen;

impl Codegen {
    pub fn gen_block(&mut self, block: &Block, builder: &mut FunctionBuilder) -> Value {
        let blk = builder.create_block();
        builder.switch_to_block(blk);
        builder.seal_block(blk);
        for stmt in block.statements() {
            match stmt {
                crate::frontend::ast::statements::Statement::Assignment(var_name, expression) => {
                    let ty = expression.associated_type.to_cranelift();
                    let var = Variable::new(self.variables.len());
                    let val = self.gen_expression(expression, builder);
                    builder.declare_var(var, ty);
                    builder.def_var(var, val);
                    self.variables.insert(var_name.to_owned(), (ty, var));
                }
                crate::frontend::ast::statements::Statement::Return(expr) => {
                    return self.gen_expression(expr, builder);
                }
            }
        }
        self.gen_expression(&block.return_expr, builder)
    }
}
