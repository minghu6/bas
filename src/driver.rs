use std::{error::Error, path::Path, time::{SystemTime, UNIX_EPOCH}};

use m6lexerkit::SrcFileInfo;

use crate::{lexer::tokenize, parser::parse, ast_lowering::semantic_analyze};

pub struct RunCompiler {}

fn _unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}


impl RunCompiler {
    pub fn new(src: &Path) -> Result<Self, Box<dyn Error>> {
        let src = SrcFileInfo::new(src)?;

        let tokens = tokenize(&src)?;
        let tt = parse(tokens, &src)?;
        let _amod = semantic_analyze(tt, &src)?;

        Ok(Self {})
    }
}
