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

        if let Some(afn) = self.amod.afns.get(&name) {
            let params = afn.params.clone();

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
        }
        else {
            unreachable!("There must be definition by Pass1 {}", sym2str(name))
        }

        // body to stmts
        debug_assert_eq!(body[0].0, ST::Stmts);
        self.do_analyze_block_with_scope(scope_idx, body[0].1.as_tt());

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
