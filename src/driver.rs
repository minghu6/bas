use std::{
    error::Error,
    path::Path,
};

use inkwellkit::config::CompilerConfig;
use m6lexerkit::SrcFileInfo;

use crate::{
    ast_lowering::{semantic_analyze, AModExp}, codegen::gen_code, lexer::tokenize,
    parser::parse, env::boostrap_dir,
};

pub struct RunCompiler {}

impl RunCompiler {
    pub fn new<P: AsRef<Path>>(
        src: &P,
        config: CompilerConfig,
    ) -> Result<Self, Box<dyn Error>> {
        let core = Self::boot()?;

        let src = SrcFileInfo::new(src.as_ref())?;

        let tokens = tokenize(&src)?;
        let tt = parse(tokens, &src)?;
        let amod = semantic_analyze(tt, &src)?;

        gen_code(amod, config)?;

        Ok(Self {})
    }

    pub fn boot() -> Result<AModExp, Box<dyn Error>> {

        let core_path = boostrap_dir().join("core.bath");
        let core_src = SrcFileInfo::new(&core_path)?;

        let tokens = tokenize(&core_src)?;
        let tt = parse(tokens, &core_src)?;
        let amod = semantic_analyze(tt, &core_src)?;


        Ok(amod.export())
    }
}



#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use m6lexerkit::SrcFileInfo;

    use crate::{
        ast_lowering::semantic_analyze,
        codegen::{gen_code, sh_obj_config},
        lexer::tokenize,
        parser::parse,
    };

    use super::RunCompiler;


    #[test]
    fn test_compile() -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from("./examples/exp0.bath");
        let src = SrcFileInfo::new(&path).unwrap();

        let tokens = tokenize(&src)?;
        let tt = parse(tokens, &src)?;
        let amod = semantic_analyze(tt, &src)?;

        // gen_code(amod, sh_llvm_config(true))?;
        gen_code(amod, sh_obj_config(false, PathBuf::from("exp0")))?;

        Ok(())
    }


    #[test]
    fn test_boot() -> Result<(), Box<dyn std::error::Error>> {
        let _core = RunCompiler::boot().unwrap();

        Ok(())
    }
}

