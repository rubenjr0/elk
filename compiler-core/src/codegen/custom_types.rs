use crate::frontend::ast::types::CustomType;

use super::Codegen;

impl Codegen {
    pub fn gen_type_definition(&mut self, custom_type: &CustomType) {
        self.define_type(custom_type);
    }
}
