use super::SemanticAnalyzerPass2;
use crate::{
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
            let mut has_type_anno = false;

            if sns.peek().unwrap().0 == ST::Type {
                let ty = self.analyze_ty(&sns.next().unwrap().1.as_tt());
                // Need Explicit Type Annotation
                self.create_var(name, ty);
                has_type_anno = true;
            }

            if sns.peek().unwrap().0 == ST::assign {
                sns.next().unwrap();  // skip assign
                let var = self.analyze_expr(sns.next().unwrap().1.as_tt());

                if !has_type_anno {
                    self.create_var(name, var.ty.clone());
                }
                self.assign_var(name, var);
            }

            return;
        }

        unreachable!("ST: {:#?}", st);
    }
}
