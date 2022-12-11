use indexmap::IndexMap;
use inkwellkit::{get_ctx, types::RetTypeEnum};
use itertools::Itertools;
use m6lexerkit::{str2sym, Symbol};

use crate::{
    ast_lowering::{
        aty_arr_float, aty_arr_int, aty_arr_str, aty_f64, aty_i32, aty_str,
        AModExp, APriType, AType, AnExtFnDec,
    },
    codegen::CodeGen,
    name_mangling::mangling,
};


pub const ID_VEC: &str = "Vec";


////////////9////////////////////////////////////////////////////////////////////
//// Constant



////////////////////////////////////////////////////////////////////////////////
//// Implementation (ast_lowering)




////////////////////////////////////////////////////////////////////////////////
//// Implementation (codegen)

impl<'ctx> CodeGen<'ctx> {
    pub fn aty_arr_as_ret_type(
        &self,
        _aty: &APriType,
        _d: &u8,
    ) -> RetTypeEnum<'ctx> {
        RetTypeEnum::StructType(get_ctx().opaque_struct_type(ID_VEC))
    }

    // pub fn gen_aty_arr_as_basic_meta_type(
    //     &self,
    //     atys: &[APriType],
    // ) -> BasicMetadataTypeEnum<'ctx> {
    //     self.aty_arr_as_ret_type(atys).try_into().unwrap()
    // }
}



////////////////////////////////////////////////////////////////////////////////
//// Function

pub(crate) fn load_core_exp() -> AModExp {
    let fns = vec![
        def("len", &[("vec", aty_arr_int())], aty_i32(), "vec_len"),
        def("len", &[("vec", aty_arr_float())], aty_i32(), "vec_len"),
        def("len", &[("vec", aty_arr_str())], aty_i32(), "vec_len"),
        def("str", &[("val", aty_i32())], aty_str(), "stringify_i32"),
        def("str", &[("val", aty_f64())], aty_str(), "stringify_f64"),
        def("str", &[("val", aty_str())], aty_str(), "strdup"),
        def(
            "push",
            &[("vec", aty_arr_int()), ("val", aty_i32())],
            aty_i32(),
            "vec_push_i32",
        ),
        def(
            "push",
            &[("vec", aty_arr_float()), ("val", aty_f64())],
            aty_i32(),
            "vec_push_f64",
        ),
        def(
            "push",
            &[("vec", aty_arr_str()), ("val", aty_str())],
            aty_i32(),
            "vec_push_str",
        ),
    ];

    let afns: IndexMap<Symbol, AnExtFnDec> =
        fns.into_iter().map(|x| (x.name, x)).collect();

    AModExp { afns }
}


fn def(
    name: &str,
    params: &[(&str, AType)],
    ret: AType,
    sign_name: &str,
) -> AnExtFnDec {
    let tys = params.iter().map(|param| param.1.clone()).collect_vec();

    let namesym = mangling(str2sym(name), &tys);

    let params = params
        .iter()
        .map(|(formal, ty)| (str2sym(formal), ty.clone()))
        .collect_vec();

    AnExtFnDec {
        name: namesym,
        params,
        ret,
        sign_name: str2sym(sign_name),
    }
}
