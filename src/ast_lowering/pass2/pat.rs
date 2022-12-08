use m6lexerkit::Symbol;

use crate::{parser::TokenTree, ast_lowering::analyze_pat_no_top};

use super::SemanticAnalyzerPass2;



impl SemanticAnalyzerPass2 {
    pub(crate) fn analyze_pat_no_top(&mut self, tt: &TokenTree) -> Symbol {
        analyze_pat_no_top(tt)
    }
}
