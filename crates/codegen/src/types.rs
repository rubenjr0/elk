use ast::types::Type;

use crate::Generable;

impl Generable for Type {
    type Output = cranelift::prelude::Type;

    fn size(&self) -> u32 {
        match self {
            Self::I8 | Self::U8 | Self::Bool => 1,
            Self::I16 | Self::U16 => 2,
            Self::I32 | Self::U32 | Self::F32 => 4,
            Self::I64 | Self::U64 | Self::F64 | Self::Function(_) | Self::Custom(_, _) => 8,
            _ => todo!(),
        }
    }

    fn to_cranelift(&self) -> Self::Output {
        use cranelift::prelude::types as T;
        match self {
            Self::I8 | Self::U8 => T::I8,
            Self::I16 | Self::U16 => T::I16,
            Self::I32 | Self::U32 => T::I32,
            Self::I64 | Self::U64 | Self::Function(_) | Self::Custom(_, _) => T::I64,
            Self::F32 => T::F32,
            Self::F64 => T::F64,
            Self::Bool => T::I8,
            _ => todo!(),
        }
    }
}
