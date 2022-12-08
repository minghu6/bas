use std::rc::Rc;

use m6lexerkit::{Span, Symbol};

use super::{
    AFnDec, AMod, AParamPat, AType, AnalyzeResult2,
    SemanticErrorReason as R, analyze_ty, write_diagnosis,
};
use crate::{
    ast_lowering::analyze_pat_no_top,
    parser::{SyntaxType as ST, TokenTree},
};


pub(crate) struct SemanticAnalyzerPass1 {
    amod: AMod,
    tt: Rc<TokenTree>,
    diagnosis: Vec<(R, Span)>,
}

impl SemanticAnalyzerPass1 {
    pub(crate) fn new(amod: AMod, tt: Rc<TokenTree>) -> Self {
        Self {
            amod,
            tt,
            diagnosis: vec![],
        }
    }

    pub(crate) fn analyze(mut self) -> AnalyzeResult2 {
        for (ty, sn) in self.tt.clone().subs.iter() {
            if *ty == ST::Item {
                self.do_analyze_item(sn.as_tt());
            }
        }

        if self.diagnosis.is_empty() {
            Ok(self.amod)
        } else {
            Err(self.diagnosis)
        }
    }

    pub(crate) fn do_analyze_item(&mut self, tt: &TokenTree) {
        for (ty, sn) in tt.subs.iter() {
            if *ty == ST::Function {
                self.do_analyze_fn(sn.as_tt())
            }
        }
    }

    pub(crate) fn do_analyze_fn(&mut self, tt: &TokenTree) {
        // Syntax Node Stream
        let mut sns = tt.subs.iter().peekable();

        let idt = *sns.next().unwrap().1.as_tok();
        let fn_name = idt.value;

        if let Some(_afn) = self.amod.afns.get(&fn_name) {
            write_diagnosis(&mut self.diagnosis,
                R::DupItemDef {
                    name: fn_name,
                    prev: fn_name.1,
                },
                idt.span(),
            );
            return;
        }

        let params;
        if !sns.is_empty() && sns.peek().unwrap().0 == ST::FnParams {
            params = self.analyze_fn_params(sns.next().unwrap().1.as_tt());
        } else {
            params = vec![];
        }

        let ret;
        if !sns.is_empty() {
            ret = self.analyze_ty(&sns.next().unwrap().1.as_tt());
        } else {
            ret = AType::Void;
        }

        let afn = AFnDec {
            // body_idx: None,
            name: fn_name,
            params,
            ret,
        };

        self.amod.afns.insert(fn_name, afn);
    }

    pub(crate) fn analyze_fn_params(
        &mut self,
        tt: &TokenTree,
    ) -> Vec<AParamPat> {
        let mut sns = tt.subs.iter().peekable();
        let mut params = vec![];

        while !sns.is_empty() && sns.peek().unwrap().0 == ST::FnParam {
            let (param_ty, param_sn) = sns.next().unwrap();

            // println!("analyze fn_param_pats: param_sn {param_sn:#?}");

            if *param_ty == ST::id {
                write_diagnosis(
                    &mut self.diagnosis,
                    R::LackFormalParam,
                    param_sn.as_tok().span(),
                );
            }

            params.push(self.analyze_fn_param(param_sn.as_tt()));
        }

        params
    }

    pub(crate) fn analyze_fn_param(&mut self, tt: &TokenTree) -> AParamPat {

        // PatNoTop
        let formal = self.analyze_pat_no_top(tt[0].1.as_tt());
        let ty = self.analyze_ty(&tt[1].1.as_tt());

        AParamPat { formal, ty }
    }

    pub(crate) fn analyze_pat_no_top(&mut self, tt: &TokenTree) -> Symbol {
        analyze_pat_no_top(tt)
    }

    pub(crate) fn analyze_ty(&mut self, tt: &TokenTree) -> AType {
        match analyze_ty(tt) {
            Ok(aty) => {
                aty
            },
            Err(span) => {
                write_diagnosis(
                    &mut self.diagnosis,
                    R::UnkonwnType,
                    span,
                );
                AType::PH
            },
        }
    }
}
