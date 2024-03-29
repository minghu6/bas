use m6lexerkit::{lazy_static::lazy_static, str2sym, sym2str, Symbol};
use m6parserkit::Cursor;
use regex::Regex;

use super::SemanticAnalyzerPass2;
use crate::ast_lowering::ATag;
use crate::{
    ast_lowering::{
        aty_bool, aty_f64, aty_i32, APriType, ASymDef, AType, AVal, AVar,
        ConstVal, SemanticErrorReason as R,
    },
    name_mangling::mangling,
    parser::{SyntaxType as ST, TT},
};



impl SemanticAnalyzerPass2 {
    pub(crate) fn analyze_expr(&mut self, tt: &TT) -> AVar {
        if tt.len() == 3 && tt[0].0 == ST::Expr && tt[2].0 == ST::Expr {
            return self.analyze_bop_expr(tt);
        }

        // Atom Expr
        let (ty, sn) = &tt[0];
        let paren_tt = tt;
        let tt = sn.as_tt();

        match ty {
            /* ExprBlk */

            ST::IfExpr => self.analyze_if_expr(tt),
            ST::InfiLoopExpr => self.analyze_infi_loop_expr(tt),
            ST::BlockExpr => self.analyze_block_expr(tt),
            ST::GroupedExpr => self.analyze_expr(&tt[0].1.as_tt()),


            /* ExprSpan */

            ST::BreakExpr => self.analyze_break_expr(tt),
            ST::ContinueExpr => self.analyze_continue_expr(tt),
            ST::FunCallExpr => self.analyze_funcall_expr(tt),
            ST::LitExpr => self.analyze_lit_expr(tt),
            ST::PathExpr => self.analyze_path_expr(tt),
            ST::ReturnExpr => self.analyze_return_expr(tt),
            ST::SideEffectExpr => self.analyze_side_effect_expr(tt),
            ST::CmdExpr => self.analyze_cmd_expr(tt),
            ST::Expr => self.analyze_expr(tt),
            _ => unimplemented!("{:#?}", paren_tt),
        }
    }

    pub(crate) fn analyze_funcall_expr(&mut self, tt: &TT) -> AVar {
        debug_assert_eq!(tt[0].0, ST::PathExpr);
        debug_assert_eq!(tt[1].0, ST::GroupedExpr);

        let path = tt[0].1.as_tt();
        let grouped = tt[1].1.as_tt();

        /* get fn path name */
        let mut p = 0;

        let mut tag = None;
        if path[p].0 == ST::tag {
            tag = Some(self.analyze_tag(path[p].1.as_tok()));
            p += 1;
        }

        debug_assert_eq!(path[p].0, ST::PathExprSeg);
        let seg0 = &path[p].1.as_tt();
        let name_tok = seg0[0].1.as_tok();
        let base_name = name_tok.value;

        let fn_params_tt = grouped[1].1.as_tt();
        let mut param_syms = vec![];
        let mut param_tys = vec![];

        for (ty, sn) in fn_params_tt.subs.iter() {
            debug_assert_eq!(*ty, ST::PathExpr);
            let param_var = self.analyze_path_expr(sn.as_tt());
            let param_sym = self.bind_value(param_var.clone());

            param_syms.push(param_sym);
            param_tys.push(param_var.ty);
        }

        let mut use_raw = false;
        if let Some(atag) = tag {
            use_raw = matches!(atag, ATag::RAW);
        }

        let fullname;
        if use_raw {
            fullname = base_name;
        } else {
            /* name mangling */
            fullname = mangling(base_name, &param_tys);
        }

        if let Some(afndef) = self.find_func_by_name(fullname) {
            AVar::efn_call(afndef, param_syms)
        } else {
            self.write_dialogsis(
                R::NoMatchedFunc(base_name, param_tys),
                name_tok.span,
            );

            AVar::undefined()
        }
    }

    pub(crate) fn analyze_bop_expr(&mut self, tt: &TT) -> AVar {
        let mut p = 0;

        let tt1 = tt[p].1.as_tt();
        p += 1;

        let (bopty, bopsn) = &tt[p];
        let bop_tok = bopsn.as_tok();
        let span = bop_tok.span();
        p += 1;

        let tt2 = tt[p].1.as_tt();

        /* EXCLUDE ASSIGN CASE */

        if *bopty == ST::assign {
            let var;
            if tt1[0].0 != ST::PathExpr {
                self.write_dialogsis(
                    R::AssignRequireLV,
                    tt1[0].1.span()
                );

                return AVar::undefined();
            }
            else {
                var = self.analyze_path_expr(tt1[0].1.as_tt());
            }

            let value = self.analyze_expr(tt2);
            let valty = value.ty.clone();
            let mut valsym = self.bind_value(value);

            if var.ty != valty {
                if let Ok(_) = valty.try_cast(&var.ty) {
                    valsym = self.cast_val(valsym, var.ty);
                } else {
                    self.write_dialogsis(
                        R::CantCastType(valty.clone(), var.ty),
                        span,
                    );
                }
            }

            let (name, tagid) = var.val.as_var();

            return AVar {
                ty: valty,
                val: AVal::Assign(name, tagid, valsym),
            };
        }

        let var1 = self.analyze_expr(tt1);
        let sym1 = self.bind_value(var1.clone());
        let symdef1 = ASymDef::new(sym1, var1.ty.clone());

        /* TODO: Short Circuit Evaluation */
        if *bopty == ST::and {
            // if var1.val eq 0 { 0 } else { analyze_var2 }
            // if var1.val { 0 } else { analyze_var2 }

            let ifblk_idx = self.push_single_value_scope(AVar {
                ty: aty_bool(),
                val: AVal::ConstAlias(ConstVal::Bool(false)),
            });

            let var2 = self.analyze_expr(tt2);
            let elseblk_idx = self.push_single_value_scope(var2);

            let ifblk = AVal::IfBlock {
                if_exprs: vec![(sym1, ifblk_idx)],
                else_blk: Some(elseblk_idx),
            };

            return AVar {
                ty: aty_bool(),
                val: ifblk,
            };
        }

        if *bopty == ST::or {
            // if var1.val ne 0 { 1 } else { analyze_var2 }
            // or if var1.val { analyze_var2 } else { 1 }

            let var2 = self.analyze_expr(tt2);
            let ifblk_idx = self.push_single_value_scope(var2);

            let elseblk_idx = self.push_single_value_scope(AVar {
                ty: aty_bool(),
                val: AVal::ConstAlias(ConstVal::Bool(true)),
            });

            let ifblk = AVal::IfBlock {
                if_exprs: vec![(sym1, ifblk_idx)],
                else_blk: Some(elseblk_idx),
            };

            return AVar {
                ty: aty_bool(),
                val: ifblk,
            };
        }

        let var2 = self.analyze_expr(tt2);
        let sym2 = self.bind_value(var2.clone());
        let symdef2 = ASymDef::new(sym2, var2.ty.clone());

        let (res_symdef1, res_symdef2) =
            self.lift_tys_or_diagnose(*bopty, symdef1, symdef2, span);

        let var1_sym = res_symdef1.name;
        let var2_sym = res_symdef2.name;

        let retval = AVal::BOpExpr {
            op: bopty.clone(),
            operands: vec![var1_sym, var2_sym],
        };

        return AVar {
            ty: res_symdef1.ty.clone(),
            val: retval,
        };
    }

    pub(crate) fn analyze_lit_expr(&mut self, tt: &TT) -> AVar {
        let (st, sn) = &tt[0];

        let tok = sn.as_tok();
        let mut tokv = sym2str(tok.value);

        let ty;
        let val;

        match st {
            ST::lit_char => {
                todo!()
            }
            ST::lit_str => {
                ty = AType::Pri(APriType::Ptr);
                val = AVal::ConstAlias(ConstVal::Str(tok.value));
            }
            ST::lit_rawstr => {
                todo!()
            }
            ST::lit_int => {
                let is_neg = if tokv.starts_with("-") { true } else { false };

                // Handle Hex Number Literal
                let i32val = if tokv.contains("0x") {
                    if tokv.starts_with("-") {
                        tokv = tokv.trim_start_matches("-").to_string();
                    } else if tokv.starts_with("+") {
                        tokv = tokv.trim_start_matches("+").to_string();
                    }

                    tokv = tokv.trim_start_matches("0x").to_string();

                    if is_neg {
                        -i32::from_str_radix(&tokv, 16).unwrap()
                    } else {
                        i32::from_str_radix(&tokv, 16).unwrap()
                    }
                } else {
                    tokv.parse::<i32>().unwrap()
                };

                ty = aty_i32();
                val = AVal::ConstAlias(ConstVal::Int(i32val));
            }
            ST::lit_float => {
                let is_neg = if tokv.starts_with("-") { true } else { false };

                let purestr =
                    tokv.trim_start_matches("-").trim_start_matches("+");

                let f64val = match purestr.parse::<f64>() {
                    Ok(res) => {
                        if is_neg {
                            -res
                        } else {
                            res
                        }
                    }
                    Err(_err) => unreachable!(),
                };

                ty = aty_f64();
                val = AVal::ConstAlias(ConstVal::Float(f64val));
            }
            ST::lit_bool => {
                let boolval = tokv == "true";

                ty = aty_bool();
                val = AVal::ConstAlias(ConstVal::Bool(boolval));
            }
            _ => unreachable!(),
        }

        AVar { ty, val }
    }

    // pub(crate) fn analyze_path_seg(&mut self, tt: &TokenTree) -> Symbol {
    //     tt.subs[0].1.as_tok().value
    // }

    pub(crate) fn analyze_path_expr(&mut self, tt: &TT) -> AVar {
        let p = 0;

        debug_assert_eq!(tt[p].0, ST::PathExprSeg);

        let seg0 = &tt[p].1.as_tt();

        // analyze path_expr_seg
        let idtok = seg0[0].1.as_tok();
        let id = idtok.value;

        self.find_explicit_sym_or_diagnose(id, idtok.span)
    }

    pub(crate) fn analyze_side_effect_expr(&mut self, tt: &TT) -> AVar {
        let subs = &tt.subs;

        debug_assert_eq!(subs.len(), 2);

        let idt;
        let op;
        let fst_get;

        if subs[0].0 == ST::inc || subs[0].0 == ST::dec {
            idt = *subs[1].1.as_tok();
            op = subs[0].0;
            fst_get = true;
        } else if subs[1].0 == ST::inc || subs[1].0 == ST::dec {
            idt = *subs[0].1.as_tok();
            op = subs[1].0;
            fst_get = false;
        } else {
            unreachable!("subs: {:#?}", subs)
        }

        let op = if op == ST::inc { ST::add } else { ST::sub };
        let var_id = idt.value;
        let var = self.find_explicit_sym_or_diagnose(var_id, idt.span());
        let var_ty = var.ty.clone();
        let id = self.bind_value(var.clone());

        let const_ty = aty_i32();
        let const_id = self.bind_value(AVar {
            ty: const_ty.clone(),
            val: AVal::ConstAlias(ConstVal::Int(1)),
        });

        let (symdef1, symdef2) = self.lift_tys_or_diagnose(
            op,
            ASymDef::new(id, var_ty),
            ASymDef {
                name: const_id,
                ty: const_ty,
            },
            idt.span(),
        );

        let val = AVal::BOpExpr {
            op,
            operands: vec![symdef1.name, symdef2.name],
        };
        let nxt_var = AVar {
            ty: symdef1.ty.clone(),
            val,
        };
        self.assign_var(var_id, nxt_var.clone());

        if fst_get {
            var
        } else {
            nxt_var
        }
    }

    pub(crate) fn analyze_return_expr(&mut self, tt: &TT) -> AVar {
        let mut sns = tt.subs.iter().peekable();

        let (st, sn) = sns.next().unwrap();
        debug_assert_eq!(*st, ST::ret);

        if let Some((_st, sn)) = sns.next() {
            let retvar = self.analyze_expr(sn.as_tt());
            self.build_ret(retvar, sn.span())
        } else {
            self.build_ret(AVar::void(), sn.span())
        }
    }

    pub(crate) fn analyze_cmd_expr(&mut self, tt: &TT) -> AVar {
        let mut sns = tt.subs.iter();

        let (_st, sn) = sns.next().unwrap();
        let idt = *sn.as_tok();

        // extract symbol from tokv
        let syms = extract_symbol(sn.as_tok().value);
        let mut sym_syms = Vec::with_capacity(syms.len());
        let mut string_syms = Vec::with_capacity(syms.len());

        // stringlize symbol
        for sym in syms.iter() {
            let var = self.find_explicit_sym_or_diagnose(*sym, idt.span());
            if var.ty == AType::PH {
                return var;
            }
            string_syms.push(self.build_strinify_var(var, idt.span()));
            sym_syms.push(self.build_const_str(*sym));
        }

        // string replace
        let arg0 = self.build_const_str(sn.as_tok().value);
        let arg1 = self.build_const_vec_str(sym_syms);
        let arg2 = self.build_const_vec_str(string_syms);

        let cmd_fndec = self
            .find_func_by_name(str2sym("cmd_symbols_replace"))
            .unwrap();

        let cmd_sym =
            self.bind_value(cmd_fndec.fn_call_val(&[arg0, arg1, arg2]));

        let exec_fndec = self.find_func_by_name(str2sym("exec")).unwrap();

        let exec_res = self.bind_value(exec_fndec.fn_call_val(&[cmd_sym]));
        let ctlstr = self.build_const_str(str2sym("%s\n"));

        // print stdout
        let printf_fndec = self.find_func_by_name(str2sym("printf")).unwrap();

        printf_fndec.fn_call_val(&[ctlstr, exec_res])
    }

    pub(crate) fn analyze_if_expr(&mut self, tt: &TT) -> AVar {
        let mut p = Cursor::new(tt.len());
        let span = tt[*p].1.span();  // if idt

        /* skip <if> */

        p.inc();

        let mut if_exprs = vec![];
        let mut else_blk = None;

        while !p.reach_end() {
            let cond_var = self.analyze_expr(tt[*p].1.as_tt());
            let cond_sym = self.bind_value(cond_var);
            p.inc();

            let if_expr_var =
                self.analyze_block_expr(tt[*p].1.as_tt());
            let if_expr_scope_idx = if_expr_var.val.as_block_expr_idx();
            p.inc();
            if_exprs.push((cond_sym, if_expr_scope_idx));

            if !p.reach_end() {
                if tt[*p].0 == ST::r#else {
                    /* skip <else> */
                    p.inc();

                    let elsevar = self.analyze_block_expr(tt[*p].1.as_tt());
                    else_blk = Some(elsevar.val.as_block_expr_idx());
                    break;
                }
                else {
                    debug_assert_eq!(tt[*p].0, ST::r#if);
                    p.inc();
                }
            }
        }

        // Check if_exprs and else ret type
        let if_ty = &self.amod.scopes[if_exprs[0].1].as_var().ty;
        let mut conds = if_exprs.iter().skip(1);
        let mut oths = vec![];

        while !conds.is_empty() {
            let (_sym, idx) = conds.next().unwrap();
            let scope = &self.amod.scopes[*idx];

            if scope.as_var().ty != *if_ty {
                oths.push(scope.as_var().ty);
            }
        }

        if !oths.is_empty() {
            self.write_dialogsis(
                R::IncompatIfExprs {
                    if1: if_ty.clone(),
                    oths,
                },
                span,
            );
        }

        let val = AVal::IfBlock { if_exprs, else_blk };

        AVar {
            ty: if_ty.clone(),
            val,
        }
    }

    pub(crate) fn analyze_block_expr(&mut self, tt: &TT) -> AVar {
        debug_assert!(tt.len() >= 2);
        debug_assert_eq!(tt[0].0, ST::lbrace);
        debug_assert_eq!(tt[tt.len() - 1].0, ST::rbrace);

        let scope_idx = self.push_new_scope();

        self.do_analyze_block_with_scope(scope_idx, &tt[1].1.as_tt());

        AVar {
            ty: self.amod.scopes[scope_idx].as_var().ty,
            val: AVal::BlockExpr(scope_idx),
        }
    }

    pub(crate) fn analyze_infi_loop_expr(&mut self, tt: &TT) -> AVar {
        let var = self.analyze_block_expr(tt[1].1.as_tt());
        let scope_id = var.val.as_block_expr_idx();

        self.sc.push(scope_id);

        if let Some(ref avar) = self.cur_scope().break_var {
            self.cur_scope_mut().tail.ty = avar.ty.clone();
        } else {
            self.cur_scope_mut().tail.ty = AType::Never;
        }
        let ty = self.cur_scope().as_var().ty;

        self.sc.pop();

        let val = AVal::InfiLoopExpr(scope_id);

        AVar { ty, val }
    }

    pub(crate) fn analyze_break_expr(&mut self, _tt: &TT) -> AVar {
        let var = AVar {
            ty: AType::Void,
            val: AVal::Break,
        };

        self.cur_scope_mut().break_var = Some(var.clone());

        var
    }

    pub(crate) fn analyze_continue_expr(&mut self, _tt: &TT) -> AVar {
        AVar {
            ty: AType::Void,
            val: AVal::Continue,
        }
    }

    pub(crate) fn do_analyze_block_with_scope(
        &mut self,
        scope_idx: usize,
        tt: &TT,
    ) {
        self.sc.push(scope_idx);

        for (ty, sn) in tt.iter() {
            if *ty == ST::Stmt {
                self.do_analyze_stmt(sn.as_tt());
            } else if *ty == ST::Expr {
                // Stmts ret value
                self.cur_scope_mut().tail = self.analyze_expr(sn.as_tt());
                break;
            } else {
                unreachable!("{:#?}", ty)
            }
        }

        self.sc.pop();
    }

    /// Side Effect Exec
    pub(crate) fn do_analyze_expr(&mut self, tt: &TT) {
        let avar = self.analyze_expr(tt);

        self.bind_value(avar);
    }
}


lazy_static! {
    static ref SYM_PAT: Regex =
        Regex::new("\\$([[[:alpha:]]_][[:alnum:]]*)").unwrap();
}

///
/// Extract value symbol
/// "echo -n $count" => count
///
fn extract_symbol(value: Symbol) -> Vec<Symbol> {
    let mut syms = vec![];
    // let escape_char = '\\';
    // let cmds = "echo -n $count >> $aa ";
    let tokv = sym2str(value);

    for one_pat in SYM_PAT.captures_iter(&tokv) {
        let s = one_pat.get(1).unwrap().as_str();

        syms.push(s)
    }

    syms.into_iter().map(|s| str2sym(s)).collect()
}
