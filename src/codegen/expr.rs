use either::Either;
use inkwellkit::values::{BasicValueEnum, InstructionOpcode};
use inkwellkit::{FloatPredicate, IntPredicate, VMMod};
use itertools::Itertools;
use m6lexerkit::{sym2str, Symbol};

use super::CodeGen;
use crate::ast_lowering::{APriType, AType, AVal, AVar, ConstVal, MIRTy, MIR};
use crate::parser::SyntaxType as ST;



impl<'ctx> CodeGen<'ctx> {
    fn translate_mir(&mut self, mir: MIR) {
        let MIR {
            name,
            tagid,
            mirty,
            ty,
            val,
        } = mir;

        if let Some(bv) = self.translate_avar(AVar { ty, val }) {
            match mirty {
                MIRTy::ValBind => {
                    self.bind_value(name, bv);
                }
                MIRTy::VarAssign => {
                    self.assign_var((name, tagid.unwrap()), bv)
                }
            }
        }
        else {
            unreachable!("skip {name:?} for {mirty:?}")
        }
    }

    fn translate_avar(&mut self, var: AVar) -> Option<BasicValueEnum<'ctx>> {
        Some(match var.val {
            AVal::IfBlock { if_exprs, else_blk } => {
                self.translate_if(var.ty, if_exprs, else_blk)
            }
            AVal::BlockExpr(blk_idx) => {
                if let Some(bv) = self.translate_block(blk_idx) {
                    bv
                } else {
                    return None;
                }
            }
            AVal::FnParam(idx) => self.translate_fn_param(idx),
            AVal::FnCall { call_fn, args } => {
                self.translate_fn_call(call_fn, args)
            }
            AVal::BOpExpr { op, operands } => {
                self.translate_bop_expr(op, operands)
            }
            AVal::ConstAlias(const_val) => self.translate_const_val(const_val),
            AVal::Break => self.translate_break(),
            AVal::Continue => self.translate_continue(),
            AVal::Return(sym_opt) => self.translate_return(sym_opt),
            AVal::InfiLoopExpr(blk_idx) => {
                let res = self.translate_infi_loop(blk_idx);
                self.break_to = None;

                if res.is_none() {
                    return None;
                }

                res.unwrap()
            }
            AVal::TypeCast { name, ty } => self.translate_type_cast(name, ty),
            AVal::Var(sym, tagid) => self.translate_var(sym, tagid),
            AVal::Assign(sym, tagid, valsym) => {
                let bv = self.find_sym(valsym).unwrap();
                self.assign_var((sym, tagid), bv);
                bv
            },
            AVal::PH => return None,
            _ => unreachable!("{:#?}", var),
        })
    }

    /// load local variable value
    fn translate_var(
        &self,
        sym: Symbol,
        tagid: usize,
    ) -> BasicValueEnum<'ctx> {
        if let Some(ptrval) = self.fn_alloc.get(&(sym, tagid)) {
            self.builder.build_load(*ptrval, "")
        } else {
            unreachable!("Bug: escaped symbol: {sym:?}")
        }
    }

    fn translate_bop_expr(
        &self,
        op: ST,
        operands: Vec<Symbol>,
    ) -> BasicValueEnum<'ctx> {
        debug_assert_eq!(operands.len(), 2);

        let ope1st = self.find_sym(operands[0]).unwrap();
        let ope2nd = self.find_sym(operands[1]).unwrap();

        match op {
            ST::add => {
                if ope1st.is_int_value() {
                    let operand1 = ope1st.into_int_value();
                    let operand2 = ope2nd.into_int_value();
                    self.builder.build_int_add(operand1, operand2, "").into()
                } else if ope1st.is_float_value() {
                    let operand1 = ope1st.into_float_value();
                    let operand2 = ope2nd.into_float_value();
                    self.builder.build_float_add(operand1, operand2, "").into()
                } else {
                    unimplemented!("op1st: {:?}", ope1st)
                }
            }
            ST::sub => {
                if ope1st.is_int_value() {
                    let operand1 = ope1st.into_int_value();
                    let operand2 = ope2nd.into_int_value();
                    self.builder.build_int_sub(operand1, operand2, "").into()
                } else if ope1st.is_float_value() {
                    let operand1 = ope1st.into_float_value();
                    let operand2 = ope2nd.into_float_value();
                    self.builder.build_float_sub(operand1, operand2, "").into()
                } else {
                    unimplemented!("op1st: {:?}", ope1st)
                }
            }
            ST::mul => {
                if ope1st.is_int_value() {
                    let operand1 = ope1st.into_int_value();
                    let operand2 = ope2nd.into_int_value();
                    self.builder.build_int_mul(operand1, operand2, "").into()
                } else if ope1st.is_float_value() {
                    let operand1 = ope1st.into_float_value();
                    let operand2 = ope2nd.into_float_value();
                    self.builder.build_float_mul(operand1, operand2, "").into()
                } else {
                    unimplemented!("op1st: {:?}", ope1st)
                }
            }
            ST::div => {
                if ope1st.is_int_value() {
                    let operand1 = ope1st.into_int_value();
                    let operand2 = ope2nd.into_int_value();
                    self.builder
                        .build_int_signed_div(operand1, operand2, "")
                        .into()
                } else if ope1st.is_float_value() {
                    let operand1 = ope1st.into_float_value();
                    let operand2 = ope2nd.into_float_value();
                    self.builder.build_float_div(operand1, operand2, "").into()
                } else {
                    unimplemented!("op1st: {:?}", ope1st)
                }
            }
            ST::gt | ST::ge | ST::lt | ST::le => {
                if ope1st.is_int_value() {
                    let operand1 = ope1st.into_int_value();
                    let operand2 = ope2nd.into_int_value();
                    let int_pred = match op {
                        ST::gt => IntPredicate::SGT,
                        ST::ge => IntPredicate::SGE,
                        ST::lt => IntPredicate::SLT,
                        ST::le => IntPredicate::SLE,
                        _ => unreachable!(),
                    };
                    self.builder
                        .build_int_compare(int_pred, operand1, operand2, "")
                        .into()
                } else if ope1st.is_float_value() {
                    let operand1 = ope1st.into_float_value();
                    let operand2 = ope2nd.into_float_value();
                    let float_pred = match op {
                        ST::gt => FloatPredicate::OGT,
                        ST::ge => FloatPredicate::OGE,
                        ST::lt => FloatPredicate::ULT,
                        ST::le => FloatPredicate::ULE,
                        _ => unreachable!(),
                    };
                    self.builder
                        .build_float_compare(
                            float_pred, operand1, operand2, "",
                        )
                        .into()
                } else {
                    unimplemented!("op1st: {:?}", ope1st)
                }
            }

            _ => unreachable!("{:#?}; {:#?}", op, operands),
        }
    }


    fn translate_type_cast(
        &self,
        name: Symbol,
        ty: AType,
    ) -> BasicValueEnum<'ctx> {
        let bv = self.find_sym(name).unwrap();

        if bv.is_int_value() {
            match ty {
                AType::Pri(pri) => match pri {
                    APriType::Float(_) => self.builder.build_cast(
                        InstructionOpcode::SIToFP,
                        bv,
                        pri.as_float_ty(),
                        "",
                    ),
                    APriType::Int(_swidth) => bv,
                    APriType::Ptr => todo!(),
                    APriType::OpaqueStruct(_) => unreachable!(),
                },
                _ => unreachable!(),
            }
        } else if bv.is_float_value() {
            match ty {
                AType::Pri(pri) => match pri {
                    APriType::Float(_) => bv,
                    APriType::Int(swidth) => self.builder.build_cast(
                        if swidth > 0 {
                            InstructionOpcode::FPToUI
                        } else {
                            InstructionOpcode::FPToSI
                        },
                        bv,
                        pri.as_int_ty(),
                        "",
                    ),
                    APriType::Ptr => todo!(),
                    APriType::OpaqueStruct(_) => unreachable!(),
                },
                _ => unreachable!(),
            }
        } else {
            unreachable!("{:#?}", bv)
        }
    }


    fn translate_const_val(
        &self,
        const_val: ConstVal,
    ) -> BasicValueEnum<'ctx> {
        match const_val {
            ConstVal::Int(val) => self.vmmod.i32(val).into(),
            ConstVal::Float(val) => self.vmmod.f64(val).into(),
            ConstVal::Str(val) => {
                let ptr =
                    self.vmmod.build_local_str(&self.builder, &sym2str(val)).0;
                ptr.into()
            }
            ConstVal::Bool(val) => self.vmmod.bool(val).into(),
        }
    }

    fn translate_return(
        &mut self,
        sym_opt: Option<Symbol>,
    ) -> BasicValueEnum<'ctx> {
        // println!("translate return {sym_opt:?}");

        if let Some(sym) = sym_opt {
            if let Some(bv) = self.find_sym(sym) {
                let cur_bb = self.builder.get_insert_block().unwrap();
                self.phi_ret.push((bv, cur_bb));
            }
            else {
                unreachable!("Bug: No {sym:?}")
            }
        } else {
            // Do nothing
        }

        let blk_last =
            self.get_fnval().unwrap().get_last_basic_block().unwrap();
        self.builder.build_unconditional_branch(blk_last);

        self.has_ret = true;

        VMMod::null()
    }

    fn translate_continue(&self) -> BasicValueEnum<'ctx> {
        let bb_cur = self.continue_to.unwrap();
        self.builder.build_unconditional_branch(bb_cur);

        VMMod::null()
    }

    fn translate_break(&mut self) -> BasicValueEnum<'ctx> {
        if self.break_to.is_none() {
            self.break_to = Some(self.insert_nonterminal_bb());
        }

        let bb_nxt = self.break_to.unwrap();
        self.link_bb(bb_nxt);

        VMMod::null()
    }

    fn translate_fn_call(
        &self,
        call_fn: Symbol,
        args: Vec<Symbol>,
    ) -> BasicValueEnum<'ctx> {
        let bv_args = args
            .into_iter()
            .map(|sym| {
                if let Some(bv) = self.find_sym(sym) {
                    bv.into()
                } else {
                    unreachable!("call {:?}, arg: {:?}", call_fn, sym)
                }
            })
            .collect_vec();

        let fnval_call = if let Some(fnval) =
            self.vmmod.module.get_function(&sym2str(call_fn))
        {
            fnval
        } else {
            unreachable!("Unknown fn call: {:?}", call_fn);
        };

        match self
            .builder
            .build_call(fnval_call, &bv_args[..], "")
            .try_as_basic_value()
        {
            Either::Left(bv) => bv,
            Either::Right(_) => VMMod::null(),
        }
    }

    fn translate_fn_param(&self, idx: u32) -> BasicValueEnum<'ctx> {
        let fnval = self.get_fnval().unwrap();
        fnval.get_nth_param(idx).unwrap()
    }

    pub(crate) fn translate_block(
        &mut self,
        blk_idx: usize,
    ) -> Option<BasicValueEnum<'ctx>> {
        self.sc.push(blk_idx);

        self.has_ret = false;

        let mirs = self.amod.scopes[blk_idx].mirs.clone();
        let ret = self.amod.scopes[blk_idx].as_var();

        for mir in mirs.into_iter() {
            self.translate_mir(mir);
        }

        // println!("before ret {ret:?}");

        let ret = self.translate_avar(ret);

        // println!("after ret {ret:?}");

        self.sc.pop();

        ret
    }

    fn translate_if(
        &mut self,
        ty: AType,
        if_exprs: Vec<(Symbol, usize)>,
        else_blk: Option<usize>,
    ) -> BasicValueEnum<'ctx> {
        let if_br_len = if_exprs.len();
        let bbs = (0..if_br_len * 2)
            .map(|_| self.insert_nonterminal_bb())
            .collect_vec();

        let mut phi_local = vec![];

        let bb_nxt = if else_blk.is_some() {
            self.insert_nonterminal_bb()
        } else {
            bbs[if_br_len * 2 - 1]
        };

        for (i, (cond_sym, blk_idx)) in if_exprs.into_iter().enumerate() {
            let cond_bv = self.find_sym(cond_sym).unwrap();
            let bb_if = bbs[i * 2];
            let bb_else = bbs[i * 2 + 1];

            self.builder.build_conditional_branch(
                cond_bv.into_int_value(),
                bb_if,
                bb_else,
            );

            // build if
            self.builder.position_at_end(bb_if);

            let bv_if = self.translate_block(blk_idx);

            if !self.has_ret {
                self.builder.build_unconditional_branch(bb_nxt);

                if !matches!(ty, AType::Void | AType::Never) {
                    phi_local.push((bv_if.unwrap(), bb_if));
                }
            }

            // build else
            self.builder.position_at_end(bb_else);
            if i == if_br_len - 1 {
                if let Some(else_idx) = else_blk {
                    let bv_else = self.translate_block(else_idx);

                    if !self.has_ret {
                        self.builder.build_unconditional_branch(bb_nxt);

                        if !matches!(ty, AType::Void | AType::Never) {
                            phi_local.push((bv_else.unwrap(), bb_else));
                        }
                    }
                } else {
                    debug_assert_eq!(bb_else, bb_nxt);
                }
            }
        }

        self.builder.position_at_end(bb_nxt);

        if !matches!(ty, AType::Void | AType::Never) {
            let bmt = self.gen_aty_as_basic_meta_type(&ty);
            let phi_ret = self.builder.build_phi(bmt, "");
            for (bv, bb) in phi_local.into_iter() {
                phi_ret.add_incoming(&[(&bv, bb)]);
            }

            phi_ret.as_basic_value()
        } else {
            VMMod::null()
        }
    }

    fn translate_infi_loop(
        &mut self,
        blk_idx: usize,
    ) -> Option<BasicValueEnum<'ctx>> {
        /* Setup loop config */
        let bb_loop = self.insert_nonterminal_bb();
        self.continue_to = Some(bb_loop);

        self.link_bb(bb_loop);

        // println!("infi loop {:#?}", self.amod.scopes[blk_idx]);

        let bv = self.translate_block(blk_idx);

        self.builder.build_unconditional_branch(bb_loop);

        bv
    }
}
