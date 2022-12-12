use indexmap::indexmap;
use inkwellkit::{config::OptLv, get_ctx, module::Linkage};
use itertools::Itertools;
use m6lexerkit::{sym2str, Symbol};

use super::CodeGen;
use crate::ast_lowering::{AParamPat, AVal, MIR, AType, AnExtFnDec};




impl<'ctx> CodeGen<'ctx> {

    pub(crate) fn gen_mod(&mut self) {
        // Generate fn declaration
        for afndec in self.amod.afns.values() {
            self.gen_fn_dec(&afndec.as_ext_fn_dec(), None);
        }
        for afndec in self.amod.efns.values() {
            self.gen_fn_dec(afndec, Some(Linkage::External));
        }
        for afndec in self.ess.afns_iter() {
            self.gen_fn_dec(afndec, Some(Linkage::External));
        }

        self.gen_items()
    }

    pub(crate) fn gen_items(&mut self) {
        for MIR {
            name: _,
            mirty: _,
            tagid: _,
            ty: _,
            val,
        } in self.root_scope().mirs.clone().into_iter()
        {
            match val {
                AVal::DefFn { name, scope_idx } => {
                    self.gen_fn_body(name, scope_idx)
                }
                _ => (),
            }
        }
    }

    pub(crate) fn gen_fn_dec(&self, afndec: &AnExtFnDec, linkage: Option<Linkage>) {
        let vm_ret = self.gen_aty_as_ret_type(&afndec.ret);
        let vm_args = afndec
            .params
            .iter()
            .map(|AParamPat { formal: _, ty }| {
                self.gen_aty_as_basic_meta_type(ty)
            })
            .collect_vec();

        let fn_t = vm_ret.fn_type(&vm_args, false);

        self.vmmod
            .module
            .add_function(&sym2str(afndec.symbol_name), fn_t, linkage);
    }

    /// Name is symbol name
    pub(super) fn gen_fn_body(&mut self, name: Symbol, scope_idx: usize) {
        let module = &self.vmmod.module;
        let ctx = get_ctx();

        // create fn val
        let fn_val = module.get_function(&sym2str(name)).unwrap();
        let blk_fn_0 = ctx.append_basic_block(fn_val, "");
        self.push_bb(scope_idx, blk_fn_0);
        self.builder.position_at_end(blk_fn_0);

        // push into fn_alloc
        let fn_alloc = self.amod.allocs.get(&name).unwrap();
        self.fn_alloc = indexmap! {};

        for ((sym, tagid), ty) in fn_alloc.iter() {
            let var = self.builder.build_alloca(
                self.gen_aty_as_basic_meta_type(ty),
                &format!("{:#?}#{}", sym, tagid)
            );
            self.fn_alloc.insert((*sym, *tagid), var);
        }

        // // push into fn params
        // if let Some(_afndec) = self.amod.in_mod_find_funsym(name) {

        // }
        // else {
        //     unreachable!("{}", sym2str(name))
        // }

        // set terminator
        let bb_terminal = self.insert_terminal_bb(fn_val);

        self.phi_ret.clear();

        self.translate_block(scope_idx);

        // build terminal basick block
        self.builder.position_at_end(bb_terminal);
        let afndec = self.amod.afns.get(&name).unwrap();

        let ret = if self.phi_ret.is_empty() {
            None
        }
        else if self.phi_ret.len() == 1 {
            Some(self.phi_ret[0].0)
        }
        else {
            if matches!(afndec.ret, AType::Void) {
                unreachable!("Unexpected Non Void Return {:?}: {:#?}",name, self.phi_ret);
            }

            let ty = self.gen_aty_as_basic_meta_type(&afndec.ret);

            let phi_ret = self.builder.build_phi(
                ty,
                ""
            );
            for (bv, bb) in self.phi_ret.iter() {
                phi_ret.add_incoming(&[(bv, *bb)]);
            }

            Some(phi_ret.as_basic_value())
        };

        self.builder.build_return(ret);

        if fn_val.verify(true) {
            if self.config.optlv != OptLv::Debug {
                self.fpm.run_on(&fn_val);
            }
        }
    }
}
