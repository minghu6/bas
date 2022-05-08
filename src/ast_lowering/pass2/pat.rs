use m6lexerkit::Symbol;

use crate::parser::TokenTree;

use super::SemanticAnalyzerPass2;



impl SemanticAnalyzerPass2 {
    pub(crate) fn analyze_pat_no_top(&mut self, tt: &TokenTree) -> Symbol {
        let id = tt.subs[0].1.as_tok();

        id.name
    }
}