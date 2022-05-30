use std::error::Error;

use indexmap::IndexMap;
use inkwellkit::{
    basic_block::BasicBlock,
    builder::Builder,
    config::*,
    get_ctx,
    passes::PassManager,
    support::LLVMString,
    values::{BasicValueEnum, FunctionValue},
    VMMod,
};
use m6coll::Entry;
use m6lexerkit::{sym2str, Symbol};

use crate::ast_lowering::{AMod, AScope};

pub(crate) mod expr;
pub(crate) mod include;
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
    config: CompilerConfig,
    blks: Vec<LogicBlock<'ctx>>, // Scope - Basic Block Bindings
    fpm: PassManager<FunctionValue<'ctx>>,
    sc: Vec<usize>,
    break_to: Option<BasicBlock<'ctx>>, // loop next
    continue_to: Option<BasicBlock<'ctx>>,
    builder: Builder<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn new(amod: AMod, config: CompilerConfig) -> Self {
        let vmmod = VMMod::new(&sym2str(amod.name));
        let blks = amod.scopes.iter().map(|ascope| ascope.into()).collect();
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

        Self {
            vmmod,
            amod,
            config,
            blks,
            fpm,
            sc: vec![0],
            break_to: None,
            continue_to: None,
            builder: VMMod::get_builder(),
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

    pub(crate) fn find_sym(
        &self,
        sym: &Symbol,
    ) -> Option<BasicValueEnum<'ctx>> {
        let mut lblk = self.cur_blk();

        loop {
            if let Some(res) = lblk.in_scope_find_sym(sym) {
                break Some(res);
            } else if let Some(paren_idx) = lblk.paren {
                lblk = &self.blks[paren_idx];
            } else {
                break None;
            }
        }
    }

    pub(crate) fn bind_bv(&mut self, sym: Symbol, bv: BasicValueEnum<'ctx>) {
        self.cur_blk_mut().bind_sym(sym, bv);
    }

    pub(crate) fn push_bb(&mut self, scope_idx: usize, bb: BasicBlock<'ctx>) {
        self.blks[scope_idx].bbs.push(bb);
    }

    /// Append BasicBlock into the current logic block
    pub(crate) fn append_bb(&mut self) -> BasicBlock<'ctx> {
        let fn_val = self.get_fnval().unwrap();
        let blk = get_ctx().append_basic_block(fn_val, "");
        self.push_bb(*self.sc.last().unwrap(), blk);
        blk
    }

    pub(crate) fn get_fnval(&self) -> Option<FunctionValue<'ctx>> {
        if let Some(blk) = self.cur_blk().bbs.last() {
            blk.get_parent()
        } else {
            None
        }
    }

    /// Build Jump Instuction to the BB and position the builder to it.
    pub(crate) fn link_bb(&self, bb: BasicBlock<'ctx>) {
        self.builder.build_unconditional_branch(bb);
        self.builder.position_at_end(bb);
    }

    pub(crate) fn gen(&mut self) -> CodeGenResult {
        // self.vmmod.include_all()
        self.gen_items();
        self.gen_file()
    }

    // fn print_code(&self) -> {
    // }
}

#[derive(Debug)]
pub(crate) struct LogicBlock<'ctx> {
    pub(crate) paren: Option<usize>,
    pub(crate) bbs: Vec<BasicBlock<'ctx>>,
    pub(crate) explicit_bindings: Vec<Entry<Symbol, BasicValueEnum<'ctx>>>,
    pub(crate) implicit_bindings: IndexMap<Symbol, BasicValueEnum<'ctx>>,
}

pub(crate) fn is_implicit_sym(sym: &Symbol) -> bool {
    sym2str(*sym).starts_with("!__tmp")
}

impl<'ctx> LogicBlock<'ctx> {
    pub(crate) fn in_scope_find_sym(
        &self,
        q: &Symbol,
    ) -> Option<BasicValueEnum<'ctx>> {
        if is_implicit_sym(q) {
            // implicit_bindings
            self.implicit_bindings.get(q).cloned()
        } else {
            self.explicit_bindings
                .iter()
                .rev()
                .find(|Entry(sym, _bv)| sym == q)
                .and_then(|Entry(_sym, bv)| Some(*bv))
        }
    }

    pub(crate) fn bind_sym(&mut self, sym: Symbol, bv: BasicValueEnum<'ctx>) {
        if is_implicit_sym(&sym) {
            self.implicit_bindings.insert(sym, bv);
        } else {
            self.explicit_bindings.push(Entry(sym, bv));
        }
    }

    // /// Second Last
    // pub(crate) fn sndlast(&self) -> Option<BasicBlock<'ctx>> {
    //     let len = self.bbs.len();
    //     if len < 2 {
    //         None
    //     } else {
    //         Some(self.bbs[len - 1 - 1])
    //     }
    // }

    // pub(crate) fn last(&self) -> Option<BasicBlock<'ctx>> {
    //     let len = self.bbs.len();
    //     if len < 1 {
    //         None
    //     } else {
    //         Some(self.bbs[len - 1])
    //     }
    // }
}

impl<'ctx> From<&AScope> for LogicBlock<'ctx> {
    fn from(ascope: &AScope) -> Self {
        Self {
            paren: ascope.paren,
            bbs: vec![],
            explicit_bindings: Vec::with_capacity(
                ascope.explicit_bindings.len(),
            ),
            implicit_bindings: IndexMap::with_capacity(
                ascope.implicit_bindings.len(),
            ),
        }
    }
}


pub(crate) fn gen_code(amod: AMod, config: CompilerConfig) -> CodeGenResult {
    let mut codegen = CodeGen::new(amod, config);

    codegen.gen()
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
        emit_type: EmitType::LLVMIR,
        print_type: PrintTy::File(path),
    }
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use m6lexerkit::SrcFileInfo;

    use crate::{
        ast_lowering::semantic_analyze,
        codegen::{gen_code, sh_llvm_config},
        lexer::tokenize,
        parser::parse,
    };

    #[test]
    fn test_codegen() -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from("./examples/exp0.bath");
        let src = SrcFileInfo::new(&path).unwrap();

        // println!("{:#?}", sp_m(srcfile.get_srcstr(), SrcLoc { ln: 0, col: 0 }));

        let tokens = tokenize(&src)?;
        let tt = parse(tokens, &src)?;
        let amod = semantic_analyze(tt, &src)?;
        gen_code(amod, sh_llvm_config(true))?;

        Ok(())
    }
}
