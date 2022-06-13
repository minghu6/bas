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

    pub(crate) fn do_analyze_fn(&mut self, tt: &TokenTree) {
        let mut sns = tt.subs.iter().peekable();

        let fn_name = sns.next().unwrap().1.as_tok().value;

        if sns.peek().unwrap().0 != ST::BlockExpr {
            sns.next();
        }
        debug_assert_eq!(sns.peek().unwrap().0, ST::BlockExpr);
        let sn = &sns.next().unwrap().1;

        // There must be definition by Pass1

        // Set current fn name

        self.cur_fn = Some(fn_name);

        // Unpack Param

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

        self.do_analyze_block_with_scope(scope_idx, sn.as_tt().subs[0].1.as_tt());

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
