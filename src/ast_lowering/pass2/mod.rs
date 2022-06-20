use std::rc::Rc;

use m6coll::Entry;
use m6lexerkit::{str2sym0, Symbol, Span};

use indexmap::indexmap;

use super::{
    aty_int, aty_opaque_struct, aty_str, AMod, APriType, AScope, AType, AVal,
    AVar, AnalyzeResult2, ConstVal, DiagnosisItem2, DiagnosisType as R, MIR, ASymDef,
};
use crate::{parser::{SyntaxType as ST, TokenTree}, codegen::is_implicit_sym};

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
    sc: Vec<usize>,  // Scope Counter,
    cur_fn: Option<Symbol>,
    tt: Rc<TokenTree>,
    diagnosis: Vec<DiagnosisItem2>,
}


impl SemanticAnalyzerPass2 {
    pub(crate) fn new(mut amod: AMod, tt: Rc<TokenTree>) -> Self {
        for (sym, _afndec) in amod.afns.iter() {
            amod.allocs.insert(*sym, indexmap! {});
        }

        Self {
            amod,
            sc: vec![0],  // 0 is root
            cur_fn: None,
            tt,
            diagnosis: vec![],
        }
    }

    fn write_dialogsis(&mut self, dtype: R, span: Span) {
        self.diagnosis.push(DiagnosisItem2 { dtype, span })
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

    fn push_single_value_scope(&mut self, avar: AVar) -> usize {
        let scope_idx = self.push_new_scope();
        let scope = &mut self.amod.scopes[scope_idx];
        scope.ret = Some(avar);

        scope_idx
    }

    #[allow(unused)]
    pub(crate) fn scope_stack(&self) -> Vec<&AScope> {
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

    pub(crate) fn find_explicit_sym_ty_and_tag(&self, sym: &Symbol) -> Option<(usize, AType)> {
        let mut scope = self.cur_scope();

        loop {
            if let Some(res) = scope.in_scope_find_sym(sym) {
                break Some(res);
            } else if let Some(paren_idx) = scope.paren {
                scope = &self.amod.scopes[paren_idx];
            } else {
                // println!("sym: {:?}, span: {:?}", sym, &sym.1);
                break None;
            }
        }
    }

    pub(crate) fn find_explicit_sym_or_diagnose(
        &mut self,
        sym: Symbol,
        span: Span,
    ) -> AVar {
        if let Some((tagid, ty)) = self.find_explicit_sym_ty_and_tag(&sym) {
            AVar { ty, val: AVal::Var(sym, tagid) }
        } else {
            // println!("{:#?}", self.frame_stack());

            self.write_dialogsis(R::UnknownSymbolBinding(sym), span);

            AVar::undefined()
        }
    }

    pub(crate) fn lift_tys_or_diagnose(
        &mut self,
        op: ST,
        symdef1: ASymDef,
        symdef2: ASymDef,
        span: Span,
    ) -> (ASymDef, ASymDef) {
        // Insert Type Cast
        let sym1 = symdef1.name;
        let sym2 = symdef2.name;
        let ty1 = symdef1.ty.clone();
        let ty2 = symdef2.ty.clone();

        if let Ok(ty) = AType::lift_tys(op, ty1.clone(), ty2.clone()) {

            let mut res_sym_def1 = symdef1;
            let mut res_sym_def2 = symdef2;

            if ty1 != ty {
                let val = AVal::TypeCast { name: sym1, ty: ty.clone() };
                let res_sym1 = self.bind_value(AVar { ty: ty.clone(), val });
                res_sym_def1 = ASymDef { name: res_sym1, ty  };

            } else if ty2 != ty {
                let val = AVal::TypeCast { name: sym2, ty: ty.clone() };
                let res_sym2 = self.bind_value(AVar { ty: ty.clone(), val });
                res_sym_def2 = ASymDef { name: res_sym2, ty  };
            }

            (res_sym_def1, res_sym_def2)

        } else {
            self.write_dialogsis(
                R::IncompatiableOpType { op1: symdef1.ty, op2: symdef2.ty },
                span
            );

            (ASymDef::undefined(), ASymDef::undefined())
        }

    }

    /// For Implicit Symbol
    pub(crate) fn bind_value(&mut self, var: AVar) -> Symbol {
        let scope = self.cur_scope_mut();

        let tmp = scope.tmp_name();
        scope.mirs.push(MIR::bind_value(tmp, var));
        scope.implicit_bindings.insert(tmp, scope.mirs.len() - 1);

        tmp
    }

    /// For Explicit Symbol
    pub(crate) fn assign_var(&mut self, sym: Symbol, var: AVar) -> Symbol {
        let (tagid, ty) = self.find_explicit_sym_ty_and_tag(&sym).unwrap();

        if var.ty != ty {
            self.write_dialogsis(R::UnmatchedType(var.ty.clone(), ty), sym.1)
        }

        self.cur_scope_mut().mirs.push(MIR::assign_var(sym, tagid, var.clone()));

        sym
    }

    pub(crate) fn create_var(&mut self, sym: Symbol, ty: AType) {
        let fn_alloc
            = self.amod.allocs.get_mut(&self.cur_fn.unwrap()).unwrap();

        // get last tagid
        let mut tagid = 0;
        for (scan_sym, scan_tagid) in fn_alloc.keys().rev() {
            if sym == *scan_sym {
                tagid = scan_tagid + 1;
                break;
            }
        }

        fn_alloc.insert((sym, tagid), ty.clone());

        let scope = self.cur_scope_mut();

        scope.explicit_bindings.push(Entry(sym, (tagid, ty)));
    }

    pub(crate) fn cast_val(&mut self, varsym: Symbol, ty: AType) -> Symbol {
        let castval = AVal::TypeCast { name: varsym, ty: ty.clone() };

        self.bind_value(AVar { ty, val: castval })
    }

    pub(crate) fn build_strinify_var(
        &mut self,
        var: AVar,
        span: Span,
    ) -> Symbol {
        match var.ty {
            AType::Pri(prity) => {
                let sym = self.bind_value(var);
                let arg0 = sym;

                let val = match prity {
                    APriType::Float(8) => AVal::FnCall {
                        call_fn: str2sym0("stringify_f64"),
                        args: vec![arg0],
                    },
                    APriType::Str => AVal::FnCall {
                        call_fn: str2sym0("strdup"),
                        args: vec![arg0],
                    },
                    APriType::Int(_) => AVal::FnCall {
                        call_fn: str2sym0("stringify_i32"),
                        args: vec![arg0],
                    },
                    _ => todo!(),
                };

                self.bind_value(AVar { ty: aty_str(), val })
            }
            AType::PH => str2sym0(""),
            _ => {
                self.write_dialogsis(
                    R::UnsupportedStringifyType(var.ty.clone()),
                    span,
                );
                str2sym0("")
            }
        }
    }

    pub(super) fn build_const_str(&mut self, sym: Symbol) -> Symbol {
        let val = AVal::ConstAlias(ConstVal::Str(sym));
        let ty = aty_str();

        self.bind_value(AVar { ty, val })
    }

    pub(super) fn build_const_usize(&mut self, val: i32) -> Symbol {
        let val = AVal::ConstAlias(ConstVal::Int(val));
        let ty = aty_int(4);

        self.bind_value(AVar { ty, val })
    }

    pub(super) fn build_const_vec_str(&mut self, strs: Vec<Symbol>) -> Symbol {
        /* Create Vec */
        let cap = self.build_const_usize(strs.len().try_into().unwrap());
        let sym_vec = self.bind_value(AVar {
            ty: aty_opaque_struct("Vec"),
            val: AVal::FnCall {
                call_fn: str2sym0("vec_new_ptr"),
                args: vec![cap],
            },
        });

        for s in strs.into_iter() {
            let sym_str = if is_implicit_sym(s) {
                s
            }
            else {
                let (tagid, ty) = self.find_explicit_sym_ty_and_tag(&s).unwrap();
                self.bind_value(AVar { ty, val: AVal::Var(s, tagid) })
            };

            self.bind_value(AVar {
                ty: aty_int(-4),
                val: AVal::FnCall {
                    call_fn: str2sym0("vec_push_ptr"),
                    args: vec![sym_vec, sym_str],
                },
            });
        }

        sym_vec
    }
}
