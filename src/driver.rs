use std::{error::Error, path::Path};

use inkwellkit::config::CompilerConfig;
use m6lexerkit::SrcFileInfo;

use crate::{
    ast_lowering::{
        pass1::{SemanticAnalyzerPass1, Pass1Export}, pass2::{SemanticAnalyzerPass2, Pass2Export}, AModExp, ExtSymSet,
    },
    codegen::CodeGen,
    env::boostrap_dir,
    lexer::tokenize,
    parser::parse,
};

pub struct RunCompiler {}

impl RunCompiler {
    pub fn new<P: AsRef<Path>>(
        src: &P,
        config: CompilerConfig,
    ) -> Result<Self, Box<dyn Error>> {
        let core = Self::boot()?;

        // println!("core: {core:?}");

        let ess = ExtSymSet { mods: vec![core] };

        let src = SrcFileInfo::new(src.as_ref())?;

        let tokens = tokenize(&src)?;

        let tt = parse(tokens, &src)?;

        let Pass1Export { src, tt2, amod, ess } =
            SemanticAnalyzerPass1::run(src, tt, ess)?;

        #[allow(unused)]
        let Pass2Export { src, amod, ess } =
            SemanticAnalyzerPass2::run(src, tt2, amod, ess)?;

        CodeGen::run(amod, ess, config)?;

        Ok(Self {})
    }

    pub fn boot() -> Result<AModExp, Box<dyn Error>> {
        let core_path = boostrap_dir().join("core.bath");
        let core_src = SrcFileInfo::new(&core_path)?;

        let tokens = tokenize(&core_src)?;
        let tt = parse(tokens, &core_src)?;

        #[allow(unused)]
        let Pass1Export { src, tt2, amod, ess } =
            SemanticAnalyzerPass1::run(core_src, tt, ExtSymSet { mods: vec![] })?;

        Ok(amod.export())
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
