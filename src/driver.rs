use std::{error::Error, path::Path};

use inkwellkit::config::{self, CompilerConfig};
use m6lexerkit::SrcFileInfo;

use crate::{
    ast_lowering::{
        Pass1Export, SemanticAnalyzerPass1,
        Pass2Export, SemanticAnalyzerPass2,
        AMod, AModExp, ExtSymSet, TokenTree2,
    },
    codegen::{CodeGen, CodeGenExport},
    env::{ boostrap_dir, core_lib_path },
    lexer::tokenize,
    parser::parse,
};


pub struct RunCompiler {}


/// Used for Incrementational Compile
///
/// short as Q
pub struct Query {}
pub use Query as Q;




impl RunCompiler {
    pub fn new<P: AsRef<Path>>(
        src: &P,
        config: CompilerConfig,
    ) -> Result<Self, Box<dyn Error>> {
        let core = Self::boot()?;

        // println!("core: {core:?}");

        let ess = ExtSymSet { mods: vec![core] };

        let src = SrcFileInfo::new(src)?;

        let tokens = tokenize(&src)?;

        let tt = parse(tokens, &src)?;

        // println!("tt: {tt:#?}");

        let Pass1Export {
            src,
            tt2,
            amod,
            ess,
        } = SemanticAnalyzerPass1::run(src, tt, ess)?;

        let Pass2Export { amod, ess, .. } =
            SemanticAnalyzerPass2::run(src, tt2, amod, ess)?;

        // println!("amod: {amod:#?}");

        CodeGen::run(amod, ess, config)?;

        Ok(Self {})
    }


    pub fn boot() -> Result<AModExp, Box<dyn Error>> {
        let core_path = boostrap_dir().join("core.bath");
        let core_src = SrcFileInfo::new(&core_path)?;

        let tokens = tokenize(&core_src)?;
        let tt = parse(tokens, &core_src)?;

        // println!("core tt: {tt:#?}");

        let Pass1Export {
            src,
            tt2,
            amod,
            ess,
        } = SemanticAnalyzerPass1::run(
            core_src,
            tt,
            ExtSymSet { mods: vec![] },
        )?;

        let CodeGenExport { amod, .. } = Q::core_lib(src, tt2, amod, ess)?;

        Ok(amod.export())
    }
}


impl Query {
    pub fn core_lib(
        src: SrcFileInfo,
        tt2: TokenTree2,
        amod: AMod,
        ess: ExtSymSet,
    ) -> Result<CodeGenExport, Box<dyn Error>> {
        let core_lib_path = core_lib_path();

        // FIXME 暂时这样检测
        if core_lib_path.exists() {
            return Ok(CodeGenExport { amod, ess });
        }

        let Pass2Export { amod, ess, .. } =
            SemanticAnalyzerPass2::run(src, tt2, amod, ess)?;

        // println!("amod: {amod:#?}");

        let config = CompilerConfig {
            optlv: config::OptLv::Opt3,
            target_type: config::TargetType::ReLoc,
            // emit_type: config::EmitType::LLVMIR,
            emit_type: config::EmitType::Obj,
            print_type: config::PrintTy::File(core_lib_path),
            // print_type: config::PrintTy::StdErr,
        };

        let codegen_export = CodeGen::run(amod, ess, config)?;

        Ok(codegen_export)
    }
}



#[cfg(test)]
mod tests {
    use super::RunCompiler;

    #[test]
    fn test_boot() -> Result<(), Box<dyn std::error::Error>> {
        let _core = RunCompiler::boot().unwrap();

        Ok(())
    }
}
