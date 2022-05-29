use either::Either;
use inkwellkit::values::{BasicValueEnum, InstructionOpcode};
use inkwellkit::{FloatPredicate, IntPredicate, VMMod};
use itertools::Itertools;
use m6lexerkit::{sym2str, Symbol};

use super::CodeGen;
use crate::ast_lowering::{APriType, AType, AVal, MIR, AVar};
use crate::parser::SyntaxType as ST;




impl<'ctx> CodeGen<'ctx> {
    fn translate_mir(&mut self, mir: MIR) {
        let MIR { name, ty, val } = mir;

        let bv = self.translate_avar(AVar { ty, val });

        self.bind_bv(name, bv);
    }

    fn translate_avar(&mut self, var: AVar) -> BasicValueEnum<'ctx> {
        match var.val {
            AVal::IfBlock { if_exprs, else_blk } => {
                self.translate_if(var.ty, if_exprs, else_blk)
            }
            AVal::BlockExpr(blk_idx) => self.translate_block(blk_idx),
            AVal::FnParam(idx) => self.translate_fn_param(idx),
            AVal::FnCall { call_fn, args } => {
                self.translate_fn_call(call_fn, args)
            }
            AVal::BOpExpr { op, operands } => {
                self.translate_bop_expr(op, operands)
            }
            AVal::ConstAlias(_) => self.translate_break(),
            AVal::Break => self.translate_break(),
            AVal::Continue => self.translate_continue(),
            AVal::Return(sym_opt) => self.translate_return(sym_opt),
            AVal::InfiLoopExpr(blk_idx) => self.translate_infi_loop(blk_idx),
            AVal::TypeCast { name, ty } => self.translate_type_cast(name, ty),
            _ => unreachable!("{:#?}", var),
        }
    }

    fn translate_bop_expr(
        &self,
        op: ST,
        operands: Vec<Symbol>,
    ) -> BasicValueEnum<'ctx> {
        debug_assert_eq!(operands.len(), 2);

        let ope1st = self.find_sym(&operands[0]).unwrap();
        let ope2nd = self.find_sym(&operands[1]).unwrap();

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
                    self.builder.build_int_mul(operand1, operand2, "").into()
                } else if ope1st.is_float_value() {
                    let operand1 = ope1st.into_float_value();
                    let operand2 = ope2nd.into_float_value();
                    self.builder.build_float_mul(operand1, operand2, "").into()
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
        let bv = self.find_sym(&name).unwrap();

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
                    APriType::Str => todo!(),
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
                    APriType::Str => todo!(),
                    APriType::OpaqueStruct(_) => unreachable!(),
                },
                _ => unreachable!(),
            }
        } else {
            unreachable!("{:#?}", bv)
        }
    }


    fn translate_return(
        &self,
        sym_opt: Option<Symbol>,
    ) -> BasicValueEnum<'ctx> {
        if let Some(ref sym) = sym_opt {
            let bv = self.find_sym(sym).unwrap();
            self.builder.build_return(Some(&bv));
        } else {
            self.builder.build_return(None);
        }

        VMMod::null()
    }

    fn translate_continue(&self) -> BasicValueEnum<'ctx> {
        let bb_cur = self.continue_to.unwrap();
        self.builder.build_unconditional_branch(bb_cur);

        VMMod::null()
    }

    fn translate_break(&self) -> BasicValueEnum<'ctx> {
        let bb_nxt = self.break_to.unwrap();
        self.builder.build_unconditional_branch(bb_nxt);

        VMMod::null()
    }

    fn translate_fn_call(
        &self,
        call_fn: Symbol,
        args: Vec<Symbol>,
    ) -> BasicValueEnum<'ctx> {
        let bv_args = args
            .into_iter()
            .map(|sym| self.find_sym(&sym).unwrap().into())
            .collect_vec();

        let fnval_call =
            self.vmmod.module.get_function(&sym2str(call_fn)).unwrap();

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
    ) -> BasicValueEnum<'ctx> {
        self.sc.push(blk_idx);

        let mirs = self.amod.scopes[blk_idx].mirs.clone();
        let retopt = self.amod.scopes[blk_idx].ret.clone();

        for mir in mirs.into_iter() {
            self.translate_mir(mir);
        }
        let ret = if let Some(avar) = retopt {
            self.translate_avar(avar)
        } else {
            VMMod::null()
        };

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
        let bmt = self.gen_aty_as_basic_meta_type(&ty);
        let phi_ret = self.builder.build_phi(
            bmt,
            ""
        );

        let bbs = (0..if_br_len * 2)
            .map(|_| self.append_bb())
            .collect_vec();

        let bb_nxt = if else_blk.is_some() { self.append_bb() } else { bbs[if_br_len * 2 - 1] };

        for (i, (cond_sym, blk_idx)) in if_exprs.into_iter().enumerate() {
            let cond_bv = self.find_sym(&cond_sym).unwrap();
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
            self.builder.build_unconditional_branch(bb_nxt);
            phi_ret.add_incoming(&[(&bv_if, bb_if)]);

            // build else
            self.builder.position_at_end(bb_else);
            if i == if_br_len - 1 {
                if let Some(else_idx) = else_blk {
                    let bv_else = self.translate_block(else_idx);
                    self.builder.build_unconditional_branch(bb_nxt);
                    phi_ret.add_incoming(&[(&bv_else, bb_else)]);
                }
                else {
                    debug_assert_eq!(bb_else, bb_nxt);
                }
            }

        }

        self.builder.position_at_end(bb_nxt);

        phi_ret.as_basic_value()
    }

    fn translate_infi_loop(&mut self, blk_idx: usize) -> BasicValueEnum<'ctx> {
        /* Setup loop config */
        let bb_loop = self.append_bb();
        let bb_succr = self.append_bb();
        self.continue_to = Some(bb_loop);
        self.break_to = Some(bb_succr);

        self.link_bb(bb_loop);

        let bv = self.translate_block(blk_idx);

        self.builder.position_at_end(bb_succr);

        bv
    }
}