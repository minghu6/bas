use std::rc::Rc;

use m6lexerkit::{Token, Symbol};

use super::{AMod, AnalyzeResult2, DiagnosisItem2, AType, APriType, DiagnosisType as R, AFnDec, AParamPat};
use crate::{parser::{SyntaxType as ST, SyntaxNode as SN, TokenTree}, ast_lowering::{aty_i32, aty_f64}};


pub(crate) struct SemanticAnalyzerPass1 {
    amod: AMod,
    tt: Rc<TokenTree>,
    diagnosis: Vec<DiagnosisItem2>,
}

impl SemanticAnalyzerPass1 {
    pub(crate) fn new(amod: AMod, tt: Rc<TokenTree>) -> Self {
        Self {
            amod,
            tt,
            diagnosis: vec![],
        }
    }

    fn write_dialogsis(&mut self, dtype: R, tok: Token) {
        self.diagnosis.push(DiagnosisItem2 { dtype, tok })
    }

    pub(crate) fn analyze(mut self) -> AnalyzeResult2 {
        for (ty, sn) in self.tt.clone().subs.iter() {
            if *ty == ST::Item {
                self.do_analyze_item(sn.as_tt());
            }
        }

        if self.diagnosis.is_empty() {
            Ok(self.amod)
        }
        else {
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

        if let Some(afn) = self.amod.afns.get(&fn_name) {
            self.write_dialogsis(
                R::DupItemDef { name:fn_name, prev: afn.idt },
                idt
            );
            return;
        }

        let params;
        if !sns.is_empty() && sns.peek().unwrap().0 == ST::FnParams {
            params = self.analyze_fn_param_pats(sns.next().unwrap().1.as_tt());
        }
        else {
            params = vec![];
        }

        let ret;
        if !sns.is_empty() {
            ret = self.analyze_ty(&sns.next().unwrap().1);
        }
        else {
            ret = AType::Void;
        }

        let afn = AFnDec {
            idt,
            name: fn_name,
            params,
            ret,
        };

        // println!("{:?}", afn);

        self.amod.afns.insert(fn_name, afn);
    }

    pub(crate) fn analyze_fn_param_pats(&mut self, tt: &TokenTree) -> Vec<AParamPat> {
        let mut sns = tt.subs.iter().peekable();
        let mut params = vec![];

        while !sns.is_empty() && sns.peek().unwrap().0 == ST::FnParam {
            let (param_ty, param_sn) = sns.next().unwrap();

            if *param_ty == ST::id {
                self.write_dialogsis(R::LackFormalParam {  }, *param_sn.as_tok());
            }

            params.push(self.analyze_param_pat(param_sn.as_tt()));
        }

        params
    }

    pub(crate) fn analyze_param_pat(&mut self, tt: &TokenTree) -> AParamPat {
        let mut sns = tt.subs.iter();

        // PatNoTop
        let formal = self.analyze_pat_no_top(sns.next().unwrap().1.as_tt());
        sns.next().unwrap();
        let ty = self.analyze_ty(&sns.next().unwrap().1);

        AParamPat { formal, ty }
    }

    pub(crate) fn analyze_pat_no_top(&mut self, sn: &TokenTree) -> Symbol {
        let id = sn.subs[0].1.as_tok();

        id.name
    }

    pub(crate) fn analyze_ty(&mut self, sn: &SN) -> AType {
        match sn {
            SN::T(_) => todo!(),
            SN::E(tok) => {
                // analyze alias -- skip (inner multiple scan)
                if tok.check_value("int") {
                    return aty_i32();
                }
                if tok.check_value("float") {
                    return aty_f64();
                }
                if tok.check_value("str") {
                    return AType::Pri(APriType::Str);
                }

                todo!("ty: {}", tok);
            }
        }
    }

}
