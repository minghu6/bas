use m6coll::KVEntry;
use m6lexerkit::{Symbol, sym2str};

use crate::{
    ast_lowering::{
        AVal,
        AVar, MIR,
        SemanticAnalyzerPass2, AnItem, AType
    },
    parser::{SyntaxType as ST, TokenTree},
};



impl SemanticAnalyzerPass2 {
    pub(crate) fn do_analyze_item(&mut self, anitem: AnItem) {
        match anitem {
            AnItem::Fn { name, body } => {
                self.do_analyze_fn(name, body)
            },
        }
    }

    /// id(fn name) - FnParams(fn params) - Ret(ret type) - BlockExpr(body)
    pub(crate) fn do_analyze_fn(&mut self, name: Symbol, body: TokenTree) {
        // Set current fn name
        self.cur_fn = Some(name);

        /* Unpack Param (into body) */

        let scope_idx = self.push_new_scope();
        self.sc.push(scope_idx);

        if let Some(afn) = self.amod.afns.get(&name) {
            let params = afn.params.clone();

            for (i, param_pat) in params.into_iter().enumerate() {
                let aval = AVal::FnParam(i as u32);

                self.cur_scope_mut()
                    .explicit_bindings
                    .push(KVEntry(
                        param_pat.formal,
                        (0, AVar { ty: param_pat.ty.clone(), val: aval })
                    ));

                self.cur_scope_mut().mirs.push(MIR::bind_value(
                    param_pat.formal,
                    AVar {
                        ty: param_pat.ty.clone(),
                        val: AVal::FnParam(i as u32),
                    },
                ))
            }
        }
        else {
            unreachable!("There must be definition by Pass1 {}", sym2str(name))
        }
        self.sc.pop();

        // body to stmts
        debug_assert_eq!(body[0].0, ST::lbrace);
        debug_assert_eq!(body[1].0, ST::Stmts);

        self.do_analyze_block_with_scope(scope_idx, body[1].1.as_tt());

        /* Ensure tail return if no Never type */
        self.sc.push(scope_idx);

        if let Some(mir) = self.cur_scope_mut().mirs.last()
           && matches!(mir.val, AVal::Return(..))
        {
            // do noting
        }
        else if self.cur_scope().as_var().ty == AType::Never {
            // do nothing
        }
        else {
            let stmts = body[1].1.as_tt();
            let span;
            if let Some((_st, sn)) = stmts.subs.last() {
                span = sn.span();
            }
            else {
                span = body[0].1.as_tok().span;
            }

            let tail_return_avar = self.build_ret(
                self.cur_scope().tail.clone(),
                span
            );
            let tail_return_mir = MIR::bind_value(
                self.cur_scope().tmp_name(),
                tail_return_avar
            );
            self.cur_scope_mut().mirs.push(tail_return_mir);
        }

        self.sc.pop();


        let val = AVal::DefFn {
            name,
            scope_idx,
        };
        self.bind_value(AVar {
            ty: AType::Void,
            val,
        });

        // Unset current fn name
        self.cur_fn = None;
    }

}
