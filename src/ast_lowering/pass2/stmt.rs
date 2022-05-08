use m6coll::Entry;

use super::SemanticAnalyzerPass2;
use crate::{
    ast_lowering::MIR,
    parser::{SyntaxType as ST, TokenTree},
};


impl SemanticAnalyzerPass2 {
    pub(crate) fn do_analyze_stmt(&mut self, tt: &TokenTree) {
        let mut sns = tt.subs.iter().peekable();

        let (st, sn) = sns.next().unwrap();

        if *st == ST::Expr {
            self.do_analyze_expr(sn.as_tt());
            return;
        }

        if *st == ST::r#let {
            let name = self.analyze_pat_no_top(sns.next().unwrap().1.as_tt());
            if sns.peek().unwrap().0 != ST::assign {
                self.cur_scope_mut().mirs.push(MIR::undefined(name));
                let mir_idx = self.cur_scope().mirs.len() - 1;
                self.cur_scope_mut()
                    .explicit_bindings
                    .push(Entry(name, mir_idx));
            } else {
                sns.next().unwrap(); // skip assign
                let var = self.analyze_expr(sns.next().unwrap().1.as_tt());
                self.cur_scope_mut().mirs.push(MIR::bind(name, var));
            }
        }

        unreachable!()
    }
}
