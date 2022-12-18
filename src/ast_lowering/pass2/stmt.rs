use super::SemanticAnalyzerPass2;
use crate::parser::{ST, TT};


impl SemanticAnalyzerPass2 {
    pub(crate) fn do_analyze_stmt(&mut self, tt: &TT) {
        let mut p = 0;

        if tt[p].0 == ST::Expr {
            self.do_analyze_expr(tt[p].1.as_tt());
            return;
        }

        if tt[p].0 == ST::r#let {
            /* skip <let> */

            p += 1;

            /* get pat_no_top */

            let name = self.analyze_pat_no_top(tt[p].1.as_tt());
            let mut has_type_anno = false;
            p += 1;


            if tt[p].0 == ST::colon {
                /* skip colon */
                p += 1;

                let ty = self.analyze_ty(tt[p].1.as_tt());
                p += 1;
                // need explicit type annotation
                self.create_var(name, ty);
                has_type_anno = true;
            }

            if tt[p].0 == ST::assign {
                /* skip assign */
                p += 1;

                let var = self.analyze_expr(tt[p].1.as_tt());

                if !has_type_anno {
                    self.create_var(name, var.ty.clone());
                }
                self.assign_var(name, var);
            }

            return;
        }

        unreachable!("ST: {:#?}", tt[p].0);
    }
}

