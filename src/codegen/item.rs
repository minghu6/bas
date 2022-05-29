use inkwellkit::{get_ctx, config::OptLv};
use itertools::Itertools;
use m6lexerkit::{sym2str, Symbol};

use crate::ast_lowering::{ AFnDec, AParamPat, MIR, AVal };

use super::CodeGen;




impl<'ctx> CodeGen<'ctx> {

    pub(crate) fn gen_items(&mut self) {
        // codegen defn
        for (_fsym, afndec) in self.amod.afns.iter() {
            self.gen_fn_dec(afndec);
        }

        for MIR { name: _, ty: _, val } in self.root_scope().mirs.clone().into_iter()
        {
            match val {
                AVal::DefFn { name, scope_idx } => {
                    self.gen_fn_body(name, scope_idx)
                }
                _ => (),
            }
        }
    }

    pub(crate) fn gen_fn_dec(&self, afndec: &AFnDec) {
        let vm_ret = self.gen_aty_as_ret_type(&afndec.ret);
        let vm_args = afndec.params
            .iter()
            .map(|AParamPat { formal: _, ty }| self.gen_aty_as_basic_meta_type(ty))
            .collect_vec();

        let fn_t = vm_ret.fn_type(&vm_args, false);

        self.vmmod.module.add_function(&sym2str(afndec.name), fn_t, None);
    }

    pub(super) fn gen_fn_body(&mut self, name: Symbol, scope_idx: usize) {
        let module = &self.vmmod.module;
        let ctx = get_ctx();

        // create fn val
        let fn_val = module.get_function(&sym2str(name)).unwrap();
        let blk_fn_0 = ctx.append_basic_block(fn_val, "");
        self.push_bb(scope_idx, blk_fn_0);
        self.builder.position_at_end(blk_fn_0);

        self.translate_block(scope_idx);

        if fn_val.verify(true) {
            if self.config.optlv != OptLv::Debug {
                self.fpm.run_on(&fn_val);
            }
        }
    }

}
