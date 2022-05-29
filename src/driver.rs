use std::{error::Error, path::Path, time::{SystemTime, UNIX_EPOCH}};

use inkwellkit::config::CompilerConfig;
use m6lexerkit::SrcFileInfo;

use crate::{lexer::tokenize, parser::parse, ast_lowering::semantic_analyze, codegen::{ gen_code } };

pub struct RunCompiler {}

fn _unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

impl RunCompiler {
    pub fn new(src: &Path, config: CompilerConfig) -> Result<Self, Box<dyn Error>> {
        let src = SrcFileInfo::new(src)?;

        let tokens = tokenize(&src)?;
        let tt = parse(tokens, &src)?;
        let amod = semantic_analyze(tt, &src)?;

        gen_code(amod, config)?;

        Ok(Self {})
    }
}

