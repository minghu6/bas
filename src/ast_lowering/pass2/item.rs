use m6coll::KVEntry;

use crate::{
    ast_lowering::{
        analyze_fn_params, get_fullname_by_fn_header, AParamPat, AType, AVal,
        AVar, MIR,
        SemanticAnalyzerPass2
    },
    parser::{SyntaxType as ST, TokenTree},
};



impl SemanticAnalyzerPass2 {
    pub(crate) fn do_analyze_item(&mut self, tt: &TokenTree) {
        for (ty, sn) in tt.subs.iter() {
            if *ty == ST::Function {
                self.do_analyze_fn(sn.as_tt())
            }
        }
    }

    /// id(fn name) - FnParams(fn params) - Ret(ret type) -  -BlockExpr(body)
    pub(crate) fn do_analyze_fn(&mut self, tt: &TokenTree) {
        let fn_base_name = tt[0].1.as_tok().value;
        let params = self.analyze_fn_params(tt[1].1.as_tt());
        let fn_full_name = get_fullname_by_fn_header(fn_base_name, &params);

        let body = tt.last().1.as_tt();

        self.cur_fn = Some(fn_full_name);

        /* Unpack Param (into body) */

        let scope_idx = self.push_new_scope();

        // There must be definition by Pass1
        debug_assert!(self.amod.afns.get(&fn_full_name).is_some());

        for (i, param_pat) in params.into_iter().enumerate() {
            self.cur_scope_mut()
                .explicit_bindings
                .push(KVEntry(param_pat.formal, (0, param_pat.ty.clone())));

            self.cur_scope_mut().mirs.push(MIR::bind_value(
                param_pat.formal,
                AVar {
                    ty: param_pat.ty.clone(),
                    val: AVal::FnParam(i as u32),
                },
            ))
        }

        // body to stmts
        debug_assert_eq!(body[0].0, ST::Stmts);
        self.do_analyze_block_with_scope(scope_idx, body[0].1.as_tt());

        let val = AVal::DefFn {
            name: fn_base_name,
            scope_idx,
        };
        self.bind_value(AVar {
            ty: AType::Void,
            val,
        });

        // Unset current fn name
        self.cur_fn = None;
    }

    pub(crate) fn analyze_fn_params(
        &mut self,
        tt: &TokenTree,
    ) -> Vec<AParamPat> {
        analyze_fn_params(&mut self.cause_lists, tt)
    }
}
