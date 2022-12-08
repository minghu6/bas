use m6lexerkit::lazy_static::lazy_static;
use m6lexerkit::{str2sym0, sym2str, Symbol};
use regex::Regex;

use super::SemanticAnalyzerPass2;
use crate::ast_lowering::{aty_bool, aty_f64, aty_i32, aty_str, ASymDef};
use crate::{
    ast_lowering::{
        APriType, AType, AVal, AVar, ConstVal, SemanticErrorReason as R,
    },
    parser::{SyntaxType as ST, TokenTree},
};



impl SemanticAnalyzerPass2 {
    pub(crate) fn analyze_expr(&mut self, tt: &TokenTree) -> AVar {
        let mut sns = tt.subs.iter().peekable();

        if sns.peek().unwrap().0 == ST::OpExpr {
            let tt1 = sns.next().unwrap().1.as_tt();
            let (bopty, bopsn) = sns.next().unwrap();
            let bop_tok = bopsn.as_tok();
            let span = bop_tok.span();
            let tt2 = sns.next().unwrap().1.as_tt();

            /* EXCLUDE ASSIGN CASE */
            if *bopty == ST::assign {
                // println!("tt1: {:#?}", tt.subs[0]);
                let var = self.analyze_path_expr(tt1.subs[0].1.as_tt());

                let value = self.analyze_expr(tt2);

                let valty = value.ty.clone();
                let mut valsym = self.bind_value(value);

                if var.ty != valty {
                    if let Ok(_) = valty.try_cast(&var.ty) {
                        valsym = self.cast_val(valsym, var.ty);
                    }
                    else {
                        self.write_dialogsis(R::CantCastType(valty.clone(), var.ty), span);
                    }
                }

                let (name, tagid) = var.val.as_var();

                return AVar {
                    ty: valty,
                    val: AVal::Assign(name, tagid, valsym)
                }
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

                let ifblk = AVal::IfBlock { if_exprs: vec![(sym1, ifblk_idx)], else_blk: Some(elseblk_idx) };

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

                let ifblk = AVal::IfBlock { if_exprs: vec![(sym1, ifblk_idx)], else_blk: Some(elseblk_idx) };

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

        // Atom Expr
        let (ty, sn) = sns.next().unwrap();
        let paren_tt = tt;
        let tt = sn.as_tt();

        match ty {
            ST::IfExpr => self.analyze_if_expr(tt),
            ST::InfiLoopExpr => self.analyze_infi_loop_expr(tt),
            ST::BreakExpr => self.analyze_break_expr(tt),
            ST::ContinueExpr => self.analyze_continue_expr(tt),

            ST::GroupedExpr => self.analyze_expr(&tt.subs[0].1.as_tt()),
            ST::LitExpr => self.analyze_lit_expr(tt),
            ST::PathExpr => self.analyze_path_expr(tt),
            ST::ReturnExpr => self.analyze_return_expr(tt),
            ST::SideEffectExpr => self.analyze_side_effect_expr(tt),
            ST::CmdExpr => self.analyze_cmd_expr(tt),

            _ => unimplemented!("{:#?}", paren_tt),
        }
    }

    pub(crate) fn analyze_lit_expr(&mut self, tt: &TokenTree) -> AVar {
        let mut sns = tt.subs.iter().peekable();
        let (st, sn) = sns.next().unwrap();

        let tok = sn.as_tok();
        let mut tokv = sym2str(tok.value);

        let ty;
        let val;

        match st {
            ST::lit_char => {
                todo!()
            }
            ST::lit_str => {
                ty = AType::Pri(APriType::Str);
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

    pub(crate) fn analyze_path_expr(&mut self, tt: &TokenTree) -> AVar {
        let mut sns = tt.subs.iter().peekable();

        let tt = sns.next().unwrap().1.as_tt();

        // analyze path_expr_seg
        let idtok = tt.subs[0].1.as_tok();
        let id = idtok.value;

        self.find_explicit_sym_or_diagnose(id, id.1)
    }

    pub(crate) fn analyze_side_effect_expr(&mut self, tt: &TokenTree) -> AVar {
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

    pub(crate) fn analyze_return_expr(&mut self, tt: &TokenTree) -> AVar {
        let mut sns = tt.subs.iter().peekable();

        let val;
        if let Some((_st, sn)) = sns.next() {
            let retvar = self.analyze_expr(sn.as_tt());
            val = AVal::Return(Some(self.bind_value(retvar)));
        } else {
            val = AVal::Return(None);
        }

        AVar {
            ty: AType::Void,
            val,
        }
    }

    pub(crate) fn analyze_cmd_expr(&mut self, tt: &TokenTree) -> AVar {
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

        let val = AVal::FnCall {
            call_fn: str2sym0("cmd_symbols_replace"),
            args: vec![arg0, arg1, arg2],
        };

        let cmd_sym = self.bind_value(AVar { ty: aty_str(), val });

        let val = AVal::FnCall {
            call_fn: str2sym0("exec"),
            args: vec![cmd_sym],
        };

        // print stdout
        let ctlstr = self.build_const_str(str2sym0("%s\n"));
        let exec_res = self.bind_value(AVar { ty: aty_str(), val });

        let val = AVal::FnCall {
            call_fn: str2sym0("printf"),
            args: vec![ctlstr, exec_res],
        };

        AVar { ty: aty_str(), val }
    }

    pub(crate) fn analyze_if_expr(&mut self, tt: &TokenTree) -> AVar {
        let mut sns = tt.subs.iter().peekable();
        let span = sns.next().unwrap().1.as_tok().span(); // if idt

        let mut if_exprs = vec![];
        let mut else_blk = None;

        while !sns.is_empty() {
            let cond_var = self.analyze_expr(sns.next().unwrap().1.as_tt());
            let cond_sym = self.bind_value(cond_var);

            // dbg!(sns.peek().unwrap());

            let if_expr_var =
                self.analyze_block_expr(sns.next().unwrap().1.as_tt());
            let if_expr_scope_idx = if_expr_var.val.as_block_expr_idx();

            if_exprs.push((cond_sym, if_expr_scope_idx));

            if !sns.is_empty() && sns.peek().unwrap().0 == ST::BlockExpr {
                let elsevar =
                    self.analyze_block_expr(sns.next().unwrap().1.as_tt());
                else_blk = Some(elsevar.val.as_block_expr_idx());
                break;
            }
        }

        // Check if_exprs and else ret type
        let if_ty = &self.amod.scopes[if_exprs[0].1].ret().ty;
        let mut conds = if_exprs.iter().skip(1);
        let mut oths = vec![];

        while !conds.is_empty() {
            let (_sym, idx) = conds.next().unwrap();
            let scope = &self.amod.scopes[*idx];

            if scope.ret().ty != *if_ty {
                oths.push(scope.ret().ty);
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

    pub(crate) fn analyze_block_expr(&mut self, tt: &TokenTree) -> AVar {
        let scope_idx = self.push_new_scope();
        self.do_analyze_block_with_scope(scope_idx, &tt.subs[0].1.as_tt());

        AVar {
            ty: self.amod.scopes[scope_idx].ret().ty,
            val: AVal::BlockExpr(scope_idx),
        }
    }

    pub(crate) fn analyze_infi_loop_expr(&mut self, tt: &TokenTree) -> AVar {
        let var = self.analyze_block_expr(tt.subs[0].1.as_tt());

        let val = AVal::InfiLoopExpr(var.val.as_block_expr_idx());

        AVar {
            ty: AType::Void,
            val,
        }
    }

    pub(crate) fn analyze_break_expr(&mut self, _tt: &TokenTree) -> AVar {
        AVar {
            ty: AType::Void,
            val: AVal::Break,
        }
    }

    pub(crate) fn analyze_continue_expr(&mut self, _tt: &TokenTree) -> AVar {
        AVar {
            ty: AType::Void,
            val: AVal::Continue,
        }
    }

    pub(crate) fn do_analyze_block_with_scope(
        &mut self,
        scope_idx: usize,
        tt: &TokenTree,
    ) {
        self.sc.push(scope_idx);

        // dbg!(&tt);

        for (ty, sn) in tt.subs.iter() {
            if *ty == ST::Stmt {
                self.do_analyze_stmt(sn.as_tt());
            } else if *ty == ST::Expr {
                // Stmts ret value
                let retval = self.analyze_expr(sn.as_tt());
                self.cur_scope_mut().ret = Some(retval);
                break;
            } else {
                unreachable!("{:#?}", ty)
            }
        }

        self.sc.pop();
    }

    /// Side Effect Exec
    pub(crate) fn do_analyze_expr(&mut self, tt: &TokenTree) {
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

    syms.into_iter().map(|s| value.derive(s) ).collect()
}
