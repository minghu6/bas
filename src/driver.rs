use std::{
    error::Error,
    path::Path,
};

use inkwellkit::config::CompilerConfig;
use m6lexerkit::SrcFileInfo;

use crate::{
    ast_lowering::semantic_analyze, codegen::gen_code, lexer::tokenize,
    parser::parse,
};

pub struct RunCompiler {}

impl RunCompiler {
    pub fn new(
        src: &Path,
        config: CompilerConfig,
    ) -> Result<Self, Box<dyn Error>> {
        let src = SrcFileInfo::new(src)?;

        let tokens = tokenize(&src)?;
        let tt = parse(tokens, &src)?;
        let amod = semantic_analyze(tt, &src)?;

        gen_code(amod, config)?;

        Ok(Self {})
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


    #[test]
    fn test_compile() -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from("./examples/exp0.bath");
        let src = SrcFileInfo::new(&path).unwrap();

        let tokens = tokenize(&src)?;
        let tt = parse(tokens, &src)?;
        let amod = semantic_analyze(tt, &src)?;

        // gen_code(amod, sh_llvm_config(true))?;
        gen_code(amod, sh_obj_config(true, PathBuf::from("exp0")))?;

        Ok(())
    }
}
