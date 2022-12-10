use super::SemanticAnalyzerPass2;
use crate::{
    ast_lowering::{analyze_ty, AType},
    parser::TokenTree,
};



impl SemanticAnalyzerPass2 {
    pub(crate) fn analyze_ty(&mut self, tt: &TokenTree) -> AType {
        analyze_ty(&mut self.cause_lists, tt)
    }
}
