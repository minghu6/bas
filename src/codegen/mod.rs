use std::error::Error;

use indexmap::{indexmap, IndexMap};
use inkwellkit::{
    basic_block::BasicBlock,
    builder::Builder,
    config::*,
    get_ctx,
    passes::PassManager,
    support::LLVMString,
    values::{BasicValueEnum, FunctionValue, PointerValue},
    VMMod,
};
use m6lexerkit::{sym2str, Symbol, str2sym};

use crate::ast_lowering::{AMod, AScope, AVal, AVar, ExtSymSet};

pub(crate) mod expr;
pub(crate) mod item;
mod targets;
pub(crate) mod ty;


#[allow(unused)]
#[derive(Debug)]
pub(crate) struct CodeGenError {
    msg: String,
}

impl CodeGenError {
    pub(crate) fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_owned(),
        }
    }
}

impl From<LLVMString> for CodeGenError {
    fn from(llvmstring: LLVMString) -> Self {
        Self::new(&llvmstring.to_string())
    }
}
impl std::fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl Error for CodeGenError {}

pub(crate) type CodeGenResult = Result<(), CodeGenError>;


pub(crate) struct CodeGen<'ctx> {
    vmmod: VMMod<'ctx>,
    amod: AMod,
    ess: ExtSymSet,
    config: CompilerConfig,
    blks: Vec<LogicBlock<'ctx>>, // Scope - Basic Block Bindings
    // dyn set when codegen fn body
    fn_alloc: IndexMap<(Symbol, usize), PointerValue<'ctx>>,

    // fn_params: IndexMap<Symbol, BasicValueEnum<'ctx>>,
    fpm: PassManager<FunctionValue<'ctx>>,
    sc: Vec<usize>,

    phi_ret: Vec<(BasicValueEnum<'ctx>, BasicBlock<'ctx>)>,

    // FIXME: 应该附着在逻辑块上
    break_to: Option<BasicBlock<'ctx>>,
    continue_to: Option<BasicBlock<'ctx>>,
    has_ret: bool,

    builder: Builder<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn run(
        amod: AMod,
        ess: ExtSymSet,
        config: CompilerConfig,
    ) -> CodeGenResult {
        if matches!(config.target_type, TargetType::Bin) {
            if !amod.afns.contains_key(&str2sym("main")) {
                return Err(CodeGenError {
                    msg: format!("No entry(main) found for {:?}", amod.name),
                });
            }
        }

        let vmmod = VMMod::new(&sym2str(amod.name));
        let mut blks: Vec<LogicBlock> = amod
            .scopes
            .iter()
            .map(|ascope| LogicBlock {
                paren: ascope.paren,
                bbs: vec![],
                is_ret: false, // Be Unkonwn yet
                value_bindings: IndexMap::with_capacity(
                    ascope.implicit_bindings.len(),
                ),
            })
            .collect();

        /* Set `is_ret` attribute of LogicBlock */
        let mut retval_scope = vec![];

        for ascope in amod.scopes.iter() {
            let AVar { ty: _, val } = &ascope.tail;

            match val {
                AVal::IfBlock { if_exprs, else_blk } => {
                    for (_, idx) in if_exprs.into_iter() {
                        retval_scope.push(*idx);
                    }
                    if let Some(idx) = else_blk {
                        retval_scope.push(*idx);
                    }
                }
                AVal::InfiLoopExpr(idx) => {
                    retval_scope.push(*idx);
                }
                AVal::BlockExpr(idx) => retval_scope.push(*idx),
                _ => (),
            }
        }

        for i in retval_scope.into_iter() {
            blks[i].is_ret = true;
        }

        let fpm = PassManager::create(&vmmod.module);

        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.add_gvn_pass();
        fpm.add_cfg_simplification_pass();
        fpm.add_basic_alias_analysis_pass();
        fpm.add_promote_memory_to_register_pass();
        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.add_tail_call_elimination_pass();

        fpm.initialize();


        let mut it = Self {
            vmmod,
            amod,
            ess,
            config,
            blks,
            fn_alloc: indexmap! {},
            fpm,
            sc: vec![0],
            phi_ret: vec![],
            break_to: None,
            continue_to: None,
            has_ret: false,
            builder: VMMod::get_builder(),
        };

        match it.config.target_type {
            TargetType::Bin => {
                it.gen_mod();
                it.gen_file()?;
            },
            TargetType::ReLoc => {
                it.gen_mod();
                it.gen_file()?;
            },
            _ => unimplemented!(),
        }

        Ok(())
    }

    pub(crate) fn root_scope(&self) -> &AScope {
        &self.amod.scopes[0]
    }

    pub(crate) fn cur_blk(&self) -> &LogicBlock<'ctx> {
        &self.blks[*self.sc.last().unwrap()]
    }

    pub(crate) fn cur_blk_mut(&mut self) -> &mut LogicBlock<'ctx> {
        &mut self.blks[*self.sc.last().unwrap()]
    }

    /// Find value bind in Logic Block upwards
    pub(crate) fn find_sym(
        &self,
        sym: Symbol,
    ) -> Option<BasicValueEnum<'ctx>> {
        let mut lblk = self.cur_blk();

        loop {
            if let Some(res) = lblk.in_scope_find_val_sym(sym) {
                break Some(res);
            } else if let Some(paren_idx) = lblk.paren {
                lblk = &self.blks[paren_idx];
            } else {
                break None;
            }
        }
    }

    pub(crate) fn bind_value(
        &mut self,
        sym: Symbol,
        bv: BasicValueEnum<'ctx>,
    ) {
        self.cur_blk_mut().value_bindings.insert(sym, bv);
    }

    pub(crate) fn assign_var(
        &mut self,
        (sym, tagid): (Symbol, usize),
        bv: BasicValueEnum<'ctx>,
    ) {
        if let Some(ptr) = self.fn_alloc.get(&(sym, tagid)) {
            self.builder.build_store(*ptr, bv);
        } else {
            unreachable!("sym: {:?}, tagid: {}", sym, tagid)
        }
    }

    pub(crate) fn push_bb(&mut self, scope_idx: usize, bb: BasicBlock<'ctx>) {
        self.blks[scope_idx].bbs.push(bb);
    }

    pub(crate) fn insert_terminal_bb(
        &mut self,
        fnval: FunctionValue<'ctx>,
    ) -> BasicBlock<'ctx> {
        let blk = get_ctx().append_basic_block(fnval, "");
        self.push_bb(*self.sc.last().unwrap(), blk);
        blk
    }

    pub(crate) fn insert_nonterminal_bb(&mut self) -> BasicBlock<'ctx> {
        let fn_val = self.get_fnval().unwrap();
        let blk_last = fn_val.get_last_basic_block().unwrap();

        let blk = get_ctx().prepend_basic_block(blk_last, "");
        self.push_bb(*self.sc.last().unwrap(), blk);
        blk
    }

    pub(crate) fn get_fnval(&self) -> Option<FunctionValue<'ctx>> {
        let mut lblk = self.cur_blk();

        loop {
            if let Some(fnval) = lblk.in_scope_get_fnval() {
                break Some(fnval);
            }

            if let Some(paren_idx) = lblk.paren {
                lblk = &self.blks[paren_idx];
            } else {
                break None;
            }
        }
    }

    /// Build Jump Instuction to the BB and position the builder to it.
    pub(crate) fn link_bb(&self, bb: BasicBlock<'ctx>) {
        self.builder.build_unconditional_branch(bb);
        self.builder.position_at_end(bb);
    }
}


#[derive(Debug)]
pub(crate) struct LogicBlock<'ctx> {
    pub(crate) paren: Option<usize>,
    pub(crate) bbs: Vec<BasicBlock<'ctx>>,
    pub(crate) value_bindings: IndexMap<Symbol, BasicValueEnum<'ctx>>,
    pub(crate) is_ret: bool,
}

pub(crate) fn is_implicit_sym(sym: Symbol) -> bool {
    sym2str(sym).starts_with("!__tmp")
}

impl<'ctx> LogicBlock<'ctx> {
    pub(crate) fn in_scope_find_val_sym(
        // Value symbol or whose alias is implicit symbol
        &self,
        q: Symbol,
    ) -> Option<BasicValueEnum<'ctx>> {
        self.value_bindings.get(&q).cloned()
    }

    pub(crate) fn in_scope_get_fnval(&self) -> Option<FunctionValue<'ctx>> {
        if let Some(blk) = self.bbs.last() {
            blk.get_parent()
        } else {
            None
        }
    }
}


#[cfg(test)]
pub(crate) fn sh_llvm_config(debug: bool) -> CompilerConfig {
    CompilerConfig {
        optlv: if debug { OptLv::Debug } else { OptLv::Opt2 },
        target_type: TargetType::Bin,
        emit_type: EmitType::LLVMIR,
        print_type: PrintTy::StdErr,
    }
}


#[cfg(test)]
use std::path::PathBuf;

#[cfg(test)]
#[allow(unused)]
pub(crate) fn sh_obj_config(debug: bool, path: PathBuf) -> CompilerConfig {
    CompilerConfig {
        optlv: if debug { OptLv::Debug } else { OptLv::Opt2 },
        target_type: TargetType::Bin,
        emit_type: EmitType::Obj,
        print_type: PrintTy::File(path),
    }
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{codegen::sh_llvm_config, driver::RunCompiler};

    #[test]
    fn test_codegen() -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from("./examples/exp0.bath");

        RunCompiler::new(&path, sh_llvm_config(true))?;

        Ok(())
    }
}
