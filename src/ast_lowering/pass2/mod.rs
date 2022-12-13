use indexmap::indexmap;
use m6coll::KVEntry as Entry;
use m6lexerkit::{str2sym, sym2str, Span, SrcFileInfo, Symbol, Token};

use super::{
     analyze_pat_no_top, analyze_ty,
    aty_int, aty_str, write_diagnosis, AMod, AScope, ASymDef, AType,
    AVal, AVar, AnExtFnDec, ConstVal, ExtSymSet, SemanticError,
    SemanticErrorReason as R, MIR, TokenTree2, APriType, ATag,
};
use crate::{
    codegen::is_implicit_sym,
    name_mangling::mangling,
    parser::{SyntaxType as ST, TokenTree},
};

mod expr;
mod item;
mod stmt;


///
/// 1. Simplify the Infix Expresssion into SSA/BlockExpr
///
/// 1. Build Scope Bindings
///
///
pub struct SemanticAnalyzerPass2 {
    src: SrcFileInfo,

    amod: AMod,
    ess: ExtSymSet,
    sc: Vec<usize>, // Scope Counter,
    cur_fn: Option<Symbol>,
    cause_lists: Vec<(R, Span)>,
}


pub(crate) type Pass2Result = Result<Pass2Export, SemanticError>;


pub struct Pass2Export {
    pub src: SrcFileInfo,
    pub amod: AMod,
    pub ess: ExtSymSet
}


impl SemanticAnalyzerPass2 {
    pub(crate) fn run(
        src: SrcFileInfo,
        tt: TokenTree2,
        mut amod: AMod,
        ess: ExtSymSet,
    ) -> Pass2Result {
        for (sym, _afndec) in amod.afns.iter() {
            amod.allocs.insert(*sym, indexmap! {});
        }

        let it = Self {
            src,
            amod,
            ess,
            sc: vec![0], // 0 is root
            cur_fn: None,
            cause_lists: vec![],
        };

        it.analyze(tt)
    }

    pub(crate) fn analyze(mut self, tt: TokenTree2) -> Pass2Result {
        for anitem in tt.items.into_iter() {
            self.do_analyze_item(anitem);
        }

        if self.cause_lists.is_empty() {
            Ok(Pass2Export {
                src: self.src,
                amod: self.amod,
                ess: self.ess
            })
        } else {
            Err(SemanticError {
                cause_lists: self.cause_lists,
                src: self.src,
            })
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

    #[allow(unused)]
    pub(crate) fn find_func(
        &self,
        name: &str,
        atys: &[AType],
    ) -> Option<AnExtFnDec> {
        let fullname = mangling(str2sym(name), atys);

        self.find_func_by_name(fullname)
    }

    pub(crate) fn find_func_by_name(
        &self,
        fullname: Symbol,
    ) -> Option<AnExtFnDec> {
        if let Some(afndec) = self.amod.afns.get(&fullname) {
            Some(afndec.as_ext_fn_dec())
        } else if let Some(afndec) = self.amod.efns.get(&fullname) {
            Some(afndec.clone())
        }
        else if let Some(afndec) = self.ess.find_func_by_name(fullname) {
            Some(afndec.clone())
        } else {
            None
        }
    }

    pub(crate) fn find_explicit_sym_ty_and_tag(
        &self,
        sym: &Symbol,
    ) -> Option<(usize, AVar)> {
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
        if let Some((_tagid, avar)) = self.find_explicit_sym_ty_and_tag(&sym) {
            avar
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

    /// Create a local variable (alloc)
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
        let val = AVal::Var(sym, tagid);

        let scope = self.cur_scope_mut();

        scope.explicit_bindings.push(Entry(sym, (tagid, AVar { ty, val })));
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
            AType::PH => str2sym(""),
            _ => {
                let sym = self.bind_value(var.clone());
                let arg0 = sym;

                let fullname =
                match var.ty {
                    AType::Pri(pri) => match pri {
                        APriType::Float(_) => "stringify_f64",
                        APriType::Int(_) => "stringify_i32",
                        APriType::Ptr => "strdup",
                        APriType::OpaqueStruct(_) => todo!(),
                    },
                    AType::Arr(_, _) => todo!(),
                    AType::AA(_) => todo!(),
                    AType::Void => todo!(),
                    AType::PH => todo!(),
                };

                let fullname = str2sym(fullname);
                // let strfullname = mangling(str2sym("str"), &[var.ty.clone()]);


                let ret_var;
                if let Some(an_ext_dec) = self.find_func_by_name(fullname) {
                    ret_var = an_ext_dec.fn_call_val(&[arg0]);
                } else {
                    self.write_dialogsis(
                        R::NoMatchedFunc(fullname, vec![var.ty.clone()]),
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
        let fndec = self.find_func_by_name(
            str2sym("vec_new_ptr"),
        ).unwrap();
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
                        return str2sym("");
                    }
                }
            };

            let fndec = self.find_func_by_name(
                    str2sym("vec_push_ptr")
            ).unwrap();

            self.bind_value(fndec.fn_call_val(&[sym_vec, sym_str]));
        }

        sym_vec
    }


    ////////////////////////////////////////////////////////////////////////////////
    //// Other Analyze method

    pub(crate) fn analyze_pat_no_top(&mut self, tt: &TokenTree) -> Symbol {
        analyze_pat_no_top(tt)
    }

    pub(crate) fn analyze_ty(&mut self, tt: &TokenTree) -> AType {
        analyze_ty(&mut self.cause_lists, tt)
    }

    pub(crate) fn analyze_tag(&mut self, tok: &Token) -> ATag {
        let s = tok.value_string();

        match s.as_str() {
            "raw" => {
                ATag::RAW
            },
            _ => {
                self.write_dialogsis(
                    R::UnkonwTag,
                    tok.span
                );

                ATag::PH
            }
        }
    }
}
