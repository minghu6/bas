use crate::ast_lowering::{AType, APriType, aty_f64, aty_i32};
use crate::parser::SyntaxNode as SN;

use super::SemanticAnalyzerPass2;



impl SemanticAnalyzerPass2 {

    pub(crate) fn analyze_ty(&mut self, sn: &SN) -> AType {
        let tok_id = sn.as_tt().subs[0].1.as_tok();

        // analyze alias -- skip (inner multiple scan)
        if tok_id.check_value("int") {
            return aty_i32();
        }
        if tok_id.check_value("float") {
            return aty_f64();
        }
        if tok_id.check_value("str") {
            return AType::Pri(APriType::Str);
        }

        todo!("unknown ty: {:?}", tok_id.value);
    }
}
