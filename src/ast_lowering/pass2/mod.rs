use std::rc::Rc;

use m6lexerkit::{str2sym, Symbol, Token};

use super::{
    AMod, APriType, AScope, AType, AVal, AVar, AnalyzeResult2,
    DiagnosisItem2, DiagnosisType as R, MIR, ConstVal, aty_str, aty_int,
};
use crate::parser::{SyntaxType as ST, TokenTree};

mod expr;
mod item;
mod pat;
mod stmt;
mod ty;

///
/// 1. Simplify the Infix Expresssion into SSA/BlockExpr
///
/// 1. Build Scope Bindings
///
///
pub(crate) struct SemanticAnalyzerPass2 {
    amod: AMod,
    sc: Vec<usize>, // Scope Counter
    tt: Rc<TokenTree>,
    diagnosis: Vec<DiagnosisItem2>,
}


impl SemanticAnalyzerPass2 {
    pub(crate) fn new(amod: AMod, tt: Rc<TokenTree>) -> Self {
        Self {
            amod,
            sc: vec![0],
            tt,
            diagnosis: vec![],
        }
    }

    fn write_dialogsis(&mut self, dtype: R, tok: Token) {
        self.diagnosis.push(DiagnosisItem2 { dtype, tok })
    }

    fn cur_scope(&mut self) -> &AScope {
        &self.amod.scopes[*self.sc.last().unwrap()]
    }

    fn cur_scope_mut(&mut self) -> &mut AScope {
        &mut self.amod.scopes[*self.sc.last().unwrap()]
    }

    fn push_new_scope(&mut self) -> usize {
        let mut ascope = AScope::default();
        ascope.paren = Some(*self.sc.last().unwrap());
        self.amod.scopes.push(ascope);

        self.amod.scopes.len() - 1
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

    pub(crate) fn find_explicit_sym(&self, sym: &Symbol) -> Option<AVar> {
        // 实际上const ref， 但是Rust规则限制必须为mut（不仅是所指内容内部的变化）
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };

        let mut scope = self_mut.cur_scope_mut();

        loop {
            if let Some(res) = scope.in_scope_find_sym(sym) {
                break Some(res);
            } else if let Some(paren_idx) = scope.paren {
                scope = &mut self_mut.amod.scopes[paren_idx];
            } else {
                break None;
            }
        }
    }

    pub(crate) fn build_strinify_var(
        &mut self,
        var: AVar,
        idt: Token,
    ) -> Symbol {
        match var.ty {
            AType::Pri(prity) => {
                let sym = self.cur_scope_mut().name_var(var);
                let arg0 = sym;

                let val = match prity {
                    APriType::F64 => AVal::FnCall {
                        call_fn: str2sym("stringify_f64"),
                        args: vec![arg0],
                    },
                    APriType::Str => AVal::FnCall {
                        call_fn: str2sym("strdup"),
                        args: vec![arg0],
                    },
                    APriType::Bool | APriType::Int => AVal::FnCall {
                        call_fn: str2sym("stringify_i32"),
                        args: vec![arg0],
                    },
                };

                let mir = MIR {
                    name: sym,
                    ty: AType::Pri(APriType::Str),
                    val,
                };

                sym
            }
            AType::PH => str2sym(""),
            _ => {
                self.write_dialogsis(
                    R::UnsupportedStringifyType(var.ty.clone()),
                    idt,
                );
                str2sym("")
            }
        }
    }

    pub(super) fn build_const_str(
        &mut self,
        sym: Symbol
    ) -> Symbol {
        let val = AVal::ConstAlias(ConstVal::Str(sym));
        let ty = aty_str();

        self.cur_scope_mut().name_var(AVar { ty, val })
    }

    pub(super) fn build_const_int(
        &mut self,
        val: i32
    ) -> Symbol {
        let val = AVal::ConstAlias(ConstVal::Int(val));
        let ty = aty_int();

        self.cur_scope_mut().name_var(AVar { ty, val })
    }

    pub(super) fn build_const_vec_str(
        &mut self,
        strs: &[&str]
    ) -> Symbol {

        /* Create Vec */



    }

}
