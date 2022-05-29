use inkwellkit::
    {types::{ BasicMetadataTypeEnum, RetTypeEnum }, AddressSpace };

use inkwellkit::{ load_vm_common_ty, get_ctx };
use m6lexerkit::sym2str;

use crate::ast_lowering::{ AType, APriType };

use super::CodeGen;




impl<'ctx> CodeGen<'ctx> {

    pub(super) fn gen_aty_as_ret_type(&self, aty: &AType) -> RetTypeEnum<'ctx> {
        load_vm_common_ty!(get_ctx());

        match aty {
            AType::Pri(pri) => match pri {
                APriType::Float(len) => match len {
                    8 => f64_t.into(),
                    _ => todo!()
                },
                APriType::Int(slen) => match slen {
                    8 | -8 => i64_t.into(),
                    4 | -4 => i32_t.into(),
                    1 | -1 => i8_t.into(),
                    _ => unimplemented!("{:?}", aty)
                },
                APriType::Str => i8ptr_t.into(),
                APriType::OpaqueStruct(name) =>
                    get_ctx()
                    .opaque_struct_type(&sym2str(*name))
                    .ptr_type(AddressSpace::Generic)
                    .into(),
            },
            AType::Arr(_) => todo!(),
            AType::AA(_) => todo!(),
            AType::Void => void_t.into(),
            AType::PH => unreachable!(),
        }
    }

    pub(super) fn gen_aty_as_basic_meta_type(&self, aty: &AType) -> BasicMetadataTypeEnum<'ctx> {
        load_vm_common_ty!(get_ctx());

        match aty {
            AType::Pri(_) => self.gen_aty_as_ret_type(aty).try_into().unwrap(),
            AType::Arr(_) => todo!(),
            AType::AA(_) => todo!(),
            _ => unreachable!("{:#?}", aty),
        }
    }
}

