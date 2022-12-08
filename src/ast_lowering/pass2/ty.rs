use super::SemanticAnalyzerPass2;
use crate::{
    ast_lowering::{analyze_ty, AType, SemanticErrorReason as R},
    parser::TokenTree,
};



impl SemanticAnalyzerPass2 {
    pub(crate) fn analyze_ty(&mut self, tt: &TokenTree) -> AType {
        match analyze_ty(tt) {
            Ok(aty) => {
                aty
            },
            Err(span) => {
                self.write_dialogsis(R::UnkonwnType, span);
                AType::PH
            },
        }
    }
}
