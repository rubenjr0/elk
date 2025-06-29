use ast::{
    expressions::{AssociatedType, Expression},
    types::{CustomType, Type},
};
use cranelift::prelude::{FunctionBuilder, InstBuilder, MemFlags, Value, types};
use cranelift_module::Module;

use crate::{Codegen, Generable};

impl Generable for CustomType {
    type Output = types::Type;

    fn size(&self) -> u32 {
        todo!()
    }

    fn to_cranelift(&self) -> Self::Output {
        todo!()
    }
}

impl Codegen {
    pub fn gen_new_record_instance(
        &mut self,
        record_name: &str,
        fields: &[(String, Expression)],
        builder: &mut FunctionBuilder,
    ) -> Value {
        let ty = self.get_type(record_name).expect("Type not found");
        let size = ty.size();

        let data = cranelift::prelude::StackSlotData::new(
            cranelift::prelude::StackSlotKind::ExplicitSlot,
            size,
            0,
        );
        let ss = builder.create_sized_stack_slot(data);
        let mut offset = 0;
        fields.iter().for_each(|(_, expr)| {
            let off = offset;
            if let AssociatedType::Concrete(ty) = expr.associated_type().unwrap() {
                offset += ty.size();
            } else {
                panic!("Type not inferred");
            }
            let v = self.gen_expression(expr, builder);
            builder.ins().stack_store(v, ss, off as i32);
        });

        let ty = self.module.target_config().pointer_type();
        builder.ins().stack_addr(ty, ss, 0)
    }

    pub fn gen_record_access(
        &self,
        var_name: &str,
        field_name: &str,
        builder: &mut FunctionBuilder,
    ) -> Value {
        let (var, ty) = self.get_variable(var_name).expect("Type not found");
        let Type::Custom(type_name, _) = ty else {
            panic!("Type is not defined");
        };
        let fields = self
            .get_type(type_name)
            .and_then(|t| t.get_record_fields())
            .expect("Type is not a record");
        let mut offset = 0;
        let (field, offset) = fields
            .iter()
            .map(|f| {
                let off = offset;
                offset += f.ty().size();
                (f, off)
            })
            .find(|(f, _)| f.name() == field_name)
            .unwrap();
        let ptr = builder.use_var(*var);
        builder.ins().load(
            field.ty().to_cranelift(),
            MemFlags::new(),
            ptr,
            offset as i32,
        )
    }

    pub fn gen_new_enum_instance(
        &self,
        enum_name: &str,
        variant_name: &str,
        builder: &mut FunctionBuilder,
    ) -> Value {
        let ty = self.get_type(enum_name).expect("Enum not found");
        let (discriminant, _) = ty
            .get_enum_variants()
            .expect("Type is not an enum")
            .iter()
            .find(|(_, v)| v.name() == variant_name)
            .expect("Variant not found on enum");
        let data = cranelift::prelude::StackSlotData::new(
            cranelift::prelude::StackSlotKind::ExplicitSlot,
            ty.size(),
            0,
        );
        let ss = builder.create_sized_stack_slot(data);
        let discriminant = builder.ins().iconst(types::I8, *discriminant as i64);
        builder.ins().stack_store(discriminant, ss, 0);
        let ty = self.module.target_config().pointer_type();
        builder.ins().stack_addr(ty, ss, 0)
    }
}
