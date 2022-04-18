use std::{error::Error, path::Path, time::{SystemTime, UNIX_EPOCH}};

use m6lexerkit::SrcFileInfo;

use crate::{lexer::tokenize, parser::parse};

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
        let _tt = parse(tokens, &src)?;

        Ok(Self {})
    }
}
