use m6lexerkit::{Symbol, sym2str};

use crate::ast_lowering::{ MIR, AVal };
use crate::parser::{SyntaxType as ST, TokenTree};

use super::SemanticAnalyzerPass2;



impl SemanticAnalyzerPass2 {

    /// AScope idx
    pub(crate) fn unpack_params(&mut self, fn_name: Symbol) -> usize {
        let ascope_idx = self.push_new_scope();
        let ascope = &mut self.amod.scopes[ascope_idx];

        let afn = if let Some(afn) = self.amod.afns.get(&fn_name) {
            afn
        } else {
            unreachable!("undef fnname {}", sym2str(fn_name))
        };

        for (i, param_pat) in afn.params.iter().enumerate() {
            ascope.mirs.push(MIR {
                name: param_pat.formal,
                ty: param_pat.ty.clone(),
                val: AVal::FnParam(i),
            })
        }

        ascope_idx
    }

    pub(crate) fn do_analyze_item(&mut self, tt: &TokenTree) {
        for (ty, sn) in tt.subs.iter() {
            if *ty == ST::Function {
                self.do_analyze_fn(sn.as_tt())
            }
        }
    }

    pub(crate) fn do_analyze_fn(&mut self, tt: &TokenTree) {
        let mut sns = tt.subs.iter();

        let fn_name = sns.next().unwrap().1.as_tok().value;

        while let Some((ty, sn)) = sns.next() {
            if *ty == ST::BlockExpr {
                // There must be definition by Pass1
                let scope_idx = self.unpack_params(fn_name);

                self.do_analyze_block_with_scope(scope_idx, sn.as_tt());

                break;
            }
        }
    }

}