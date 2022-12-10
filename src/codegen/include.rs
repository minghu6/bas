use inkwellkit::{
    get_ctx, impl_fn_hdr, load_vm_common_ty,
    module::{Linkage, Module},
    AddressSpace, types::PointerType, VMMod,
};

use super::CodeGen;


impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn get_vec_t() -> PointerType<'ctx> {
        let struct_vec_t = get_ctx().opaque_struct_type("dynvec");
        struct_vec_t.ptr_type(AddressSpace::Generic)
    }

    pub(crate) fn get_ptr_t() -> PointerType<'ctx> {
        get_ctx().i8_type().ptr_type(AddressSpace::Generic)
    }

    pub(crate) fn include_core(module: &Module<'ctx>) {
        load_vm_common_ty!(get_ctx());

        VMMod::include_stdio(module);

        let vec_t = Self::get_vec_t();
        let ptr_t = Self::get_ptr_t();

        impl_fn_hdr![module |
            /* Vec */
            vec_new_i32(i32) -> vec;
            vec_push_i32(vec, i32) -> i32;
            vec_get_i32(vec, i32) -> i32;
            vec_set_i32(vec, i32, i32) -> i32;
            vec_insert_i32(vec, i32, i32) -> i32;

            vec_new_f64(i32) -> vec;
            vec_push_f64(vec, f64) -> i32;
            vec_get_f64(vec, i32) -> f64;
            vec_set_f64(vec, i32, f64) -> f64;
            vec_insert_f64(vec, i32, f64) -> f64;

            vec_new_ptr(i32) -> vec;
            vec_push_ptr(vec, ptr) -> i32;
            vec_get_ptr(vec, i32) -> ptr;
            vec_set_ptr(vec, i32, ptr) -> ptr;
            vec_insert_ptr(vec, i32, ptr) -> ptr;

            vec_len(vec) -> i32;
            vec_cap(vec) -> i32;

            /* stringify */
            stringify_i32(i32) -> i8ptr;
            stringify_f64(f64) -> i8ptr;

            /* src, syms, strs */
            cmd_symbols_replace(i8ptr, vec, vec) -> i8ptr;
            exec(i8ptr) -> i8ptr;
        ];
    }
}
