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
use m6lexerkit::{str2sym, sym2str, Symbol};

use crate::ast_lowering::{AMod, AScope, ExtSymSet};

pub(crate) mod expr;
pub(crate) mod item;
mod targets;
pub(crate) mod ty;



pub struct CodeGenExport {
    pub amod: AMod,
    pub ess: ExtSymSet,
}


#[derive(Debug)]
pub(crate) struct CodeGenError(String);



impl From<LLVMString> for CodeGenError {
    fn from(llvmstring: LLVMString) -> Self {
        CodeGenError(llvmstring.to_string())
    }
}
impl std::fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl Error for CodeGenError {}

pub(crate) type CodeGenResult2 = Result<(), CodeGenError>;

pub(crate) type CodeGenResult = Result<CodeGenExport, CodeGenError>;


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
                return Err(CodeGenError(format!(
                    "No entry(main) found for {:?}",
                    amod.name
                )));
            }
        }

        let vmmod = VMMod::new(&sym2str(amod.name));
        let blks: Vec<LogicBlock> = amod
            .scopes
            .iter()
            .map(|ascope| LogicBlock {
                paren: ascope.paren,
                bbs: vec![],

                has_ret: ascope.ret_var.is_some(),
                break_to: None,
                continue_to: None,

                value_bindings: IndexMap::with_capacity(
                    ascope.implicit_bindings.len(),
                ),
            })
            .collect();

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
            builder: VMMod::get_builder(),
        };

        it.gen_mod();
        it.gen_file().map(move |_| it.export())
    }

    pub(crate) fn export(self) -> CodeGenExport {
        CodeGenExport {
            amod: self.amod,
            ess: self.ess,
        }
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

    pub(crate) break_to: Option<BasicBlock<'ctx>>,
    pub(crate) continue_to: Option<BasicBlock<'ctx>>,

    /// There is `ret` instruction in this block, so we don't go next basicblock
    pub(crate) has_ret: bool,
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
