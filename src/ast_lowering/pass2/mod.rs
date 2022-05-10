use std::rc::Rc;

use m6coll::Entry;
use m6lexerkit::{str2sym, Symbol, Token};

use super::{
    aty_int, aty_opaque_struct, aty_str, AMod, APriType, AScope, AType, AVal,
    AVar, AnalyzeResult2, ConstVal, DiagnosisItem2, DiagnosisType as R, MIR,
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

    fn cur_scope(&self) -> &AScope {
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

    pub(crate) fn frame_stack(&self) -> Vec<&AScope> {
        let mut frames = vec![];

        let scope = self.cur_scope();
        let mut scope_idx_opt = scope.paren;

        while let Some(scope_idx) = scope_idx_opt {
            frames.push(&self.amod.scopes[scope_idx]);
            scope_idx_opt = self.amod.scopes[scope_idx].paren.clone();
        }

        frames
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
        let mut scope = self.cur_scope();

        loop {
            if let Some(res) = scope.in_scope_find_sym(sym) {
                break Some(res);
            } else if let Some(paren_idx) = scope.paren {
                scope = &self.amod.scopes[paren_idx];
            } else {
                break None;
            }
        }
    }

    pub(crate) fn find_explicit_sym_or_diagnose(
        &mut self,
        sym: Symbol,
        idt: Token,
    ) -> AVar {
        if let Some(avar) = self.find_explicit_sym(&sym) {
            avar
        } else {
            println!("{:#?}", self.frame_stack());

            self.write_dialogsis(R::UnknownSymbolBinding(sym), idt);

            AVar::undefined()
        }
    }

    pub(crate) fn lift_tys_or_diagnose(
        &mut self,
        op: ST,
        ty1: AType,
        ty2: AType,
        idt: Token,
    ) -> AType {
        if let Ok(ty) = AType::lift_tys(op, ty1.clone(), ty2.clone()) {
            ty
        } else {
            self.write_dialogsis(
                R::IncompatiableOpType { op1: ty1, op2: ty2 },
                idt
            );

            AType::PH
        }

    }

    /// For Implicit Symbol
    pub(crate) fn name_var(&mut self, var: AVar) -> Symbol {
        let scope = self.cur_scope_mut();

        let tmp = scope.tmp_name();
        scope.mirs.push(MIR::bind(tmp, var));
        scope.implicit_bindings.insert(tmp, scope.mirs.len() - 1);

        tmp
    }

    /// For Explicit Symbol
    pub(crate) fn bind_var(&mut self, sym: Symbol, var: AVar) -> Symbol {
        let scope = self.cur_scope_mut();

        scope.mirs.push(MIR::bind(sym, var));
        scope.explicit_bindings.push(Entry(sym, scope.mirs.len() - 1));

        sym
    }

    pub(crate) fn build_strinify_var(
        &mut self,
        var: AVar,
        idt: Token,
    ) -> Symbol {
        match var.ty {
            AType::Pri(prity) => {
                let sym = self.name_var(var);
                let arg0 = sym;

                let val = match prity {
                    APriType::Float(8) => AVal::FnCall {
                        call_fn: str2sym("stringify_f64"),
                        args: vec![arg0],
                    },
                    APriType::Str => AVal::FnCall {
                        call_fn: str2sym("strdup"),
                        args: vec![arg0],
                    },
                    APriType::Int(_) => AVal::FnCall {
                        call_fn: str2sym("stringify_i32"),
                        args: vec![arg0],
                    },
                    _ => todo!(),
                };

                self.name_var(AVar { ty: aty_str(), val })
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

    pub(super) fn build_const_str(&mut self, sym: Symbol) -> Symbol {
        let val = AVal::ConstAlias(ConstVal::Str(sym));
        let ty = aty_str();

        self.name_var(AVar { ty, val })
    }

    pub(super) fn build_const_int(&mut self, val: i32) -> Symbol {
        let val = AVal::ConstAlias(ConstVal::Int(val));
        let ty = aty_int(-4);

        self.name_var(AVar { ty, val })
    }

    pub(super) fn build_const_vec_str(&mut self, strs: Vec<Symbol>) -> Symbol {
        /* Create Vec */
        let cap = self.build_const_int(strs.len().try_into().unwrap());
        let sym_vec = self.name_var(AVar {
            ty: aty_opaque_struct("Vec"),
            val: AVal::FnCall {
                call_fn: str2sym("vec_new_ptr"),
                args: vec![cap],
            },
        });

        for s in strs.into_iter() {
            let sym_s = self.build_const_str(s);
            self.name_var(AVar {
                ty: aty_int(-4),
                val: AVal::FnCall {
                    call_fn: str2sym("vec_push_ptr"),
                    args: vec![sym_vec, sym_s],
                },
            });
        }

        sym_vec
    }
}
