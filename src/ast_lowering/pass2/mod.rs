use std::rc::Rc;

use indexmap::indexmap;
use m6coll::KVEntry as Entry;
use m6lexerkit::{str2sym0, sym2str, Span, Symbol};

use super::{
    aty_int, aty_str, write_diagnosis, AMod,
    AScope, ASymDef, AType, AVal, AVar, AnalyzeResult2, ConstVal,
    SemanticErrorReason as R, MIR, ESS, aty_arr_str, aty_i32,
};
use crate::{
    codegen::is_implicit_sym,
    name_mangling::mangling,
    parser::{SyntaxType as ST, TokenTree},
};

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
    sc: Vec<usize>, // Scope Counter,
    cur_fn: Option<Symbol>,
    tt: Rc<TokenTree>,
    cause_lists: Vec<(R, Span)>,
}


impl SemanticAnalyzerPass2 {
    pub(crate) fn new(mut amod: AMod, tt: Rc<TokenTree>) -> Self {
        for (sym, _afndec) in amod.afns.iter() {
            amod.allocs.insert(*sym, indexmap! {});
        }

        Self {
            amod,
            sc: vec![0], // 0 is root
            cur_fn: None,
            tt,
            cause_lists: vec![],
        }
    }

    fn write_dialogsis(&mut self, r: R, span: Span) {
        write_diagnosis(&mut self.cause_lists, r, span)
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

        if self.cause_lists.is_empty() {
            Ok(self.amod)
        } else {
            Err(self.cause_lists)
        }
    }

    pub(crate) fn find_explicit_sym_ty_and_tag(
        &self,
        sym: &Symbol,
    ) -> Option<(usize, AType)> {
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
            AVar {
                ty,
                val: AVal::Var(sym, tagid),
            }
        } else {
            // println!("{:#?}", self.frame_stack());

            self.write_dialogsis(R::UnknownSymBinding(sym), span);

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
                let val = AVal::TypeCast {
                    name: sym1,
                    ty: ty.clone(),
                };
                let res_sym1 = self.bind_value(AVar {
                    ty: ty.clone(),
                    val,
                });
                res_sym_def1 = ASymDef { name: res_sym1, ty };
            } else if ty2 != ty {
                let val = AVal::TypeCast {
                    name: sym2,
                    ty: ty.clone(),
                };
                let res_sym2 = self.bind_value(AVar {
                    ty: ty.clone(),
                    val,
                });
                res_sym_def2 = ASymDef { name: res_sym2, ty };
            }

            (res_sym_def1, res_sym_def2)
        } else {
            self.write_dialogsis(
                R::IncompatOpType {
                    op1: symdef1.ty,
                    op2: symdef2.ty,
                },
                span,
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
        if let Some((tagid, _ty)) = self.find_explicit_sym_ty_and_tag(&sym) {
            self.cur_scope_mut()
                .mirs
                .push(MIR::assign_var(sym, tagid, var));
        } else {
            unreachable!("Compiler Bug Unmatched sym {}", sym2str(sym))
        }

        sym
    }

    pub(crate) fn create_var(&mut self, sym: Symbol, ty: AType) {
        let fn_alloc =
            self.amod.allocs.get_mut(&self.cur_fn.unwrap()).unwrap();

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
        let castval = AVal::TypeCast {
            name: varsym,
            ty: ty.clone(),
        };

        self.bind_value(AVar { ty, val: castval })
    }

    pub(crate) fn build_strinify_var(
        &mut self,
        var: AVar,
        span: Span,
    ) -> Symbol {
        match var.ty {
            AType::PH => str2sym0(""),
            _ => {
                let sym = self.bind_value(var.clone());
                let arg0 = sym;

                let strfullname = mangling(str2sym0("str"), &[var.ty.clone()]);

                let ret_var;
                if let Some(an_ext_dec) = ESS.find_func_by_name(strfullname) {
                    ret_var = an_ext_dec.fn_call_val(&[arg0]);
                } else {
                    self.write_dialogsis(
                        R::NoMatchedFunc(strfullname, vec![var.ty.clone()]),
                        span,
                    );
                    ret_var = AVar::undefined()
                }

                self.bind_value(ret_var)
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
        let fndec = ESS.find_func("new_vec", &[aty_i32()]).unwrap();
        let sym_vec = self.bind_value(fndec.fn_call_val(&[cap]));

        for s in strs.into_iter() {
            let sym_str = if is_implicit_sym(s) {
                s
            } else {
                let var =
                    self.find_explicit_sym_or_diagnose(s, Span::default());

                match var.val {
                    AVal::Var(sym, _) => {
                        self.bind_value(var);
                        sym
                    }
                    _ => {
                        return str2sym0("");
                    }
                }
            };

            let fndec = ESS.find_func("push", &[aty_arr_str(), aty_str()]).unwrap();
            self.bind_value(fndec.fn_call_val(&[sym_vec, sym_str]));
        }

        sym_vec
    }
}
