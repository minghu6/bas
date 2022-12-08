use m6lexerkit::sym2str;

use super::SemanticAnalyzerPass2;
use crate::ast_lowering::{AType, AVal, AVar, MIR};
use crate::parser::{SyntaxType as ST, TokenTree};



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
        let fn_name = tt[0].1.as_tok().value;
        let body = tt.last().1.as_tt();

        // There must be definition by Pass1

        self.cur_fn = Some(fn_name);

        /* Unpack Param */

        let scope_idx = self.push_new_scope();
        let scope = &mut self.amod.scopes[scope_idx];

        let afn = if let Some(afn) = self.amod.afns.get(&fn_name) {
            afn
        } else {
            unreachable!("undef fnname {}", sym2str(fn_name))
        };

        for (i, param_pat) in afn.params.iter().enumerate() {
            scope.mirs.push(MIR::bind_value(
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
            name: fn_name,
            scope_idx,
        };
        self.bind_value(
            AVar {
                ty: AType::Void,
                val,
            },
        );

        // Unset current fn name
        self.cur_fn = None;

    }
}
