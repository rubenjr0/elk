use cranelift::prelude::{FunctionBuilder, Value};

use crate::frontend::ast::expressions::Expression;

use super::Codegen;

impl Codegen {
    pub fn gen_new_record_instance(
        &self,
        record_name: &str,
        fields: &[(String, Expression)],
        builder: &mut FunctionBuilder,
    ) -> Value {
        let record_fields = self
            .get_type(record_name)
            .and_then(|t| t.get_record_fields())
            .unwrap();

        todo!()
    }

    pub fn gen_record_access(
        &self,
        record_name: &str,
        field_name: &str,
        builder: &mut FunctionBuilder,
    ) -> Value {
        todo!()
    }
}
