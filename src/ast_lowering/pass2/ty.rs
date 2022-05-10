use crate::ast_lowering::{AType, APriType, aty_int, aty_f64, aty_i32};
use crate::parser::SyntaxNode as SN;

use super::SemanticAnalyzerPass2;



impl SemanticAnalyzerPass2 {

    pub(crate) fn analyze_ty(&mut self, sn: &SN) -> AType {
        match sn {
            SN::T(_) => todo!(),
            SN::E(tok) => {
                // analyze alias -- skip (inner multiple scan)
                if tok.check_value("int") {
                    return aty_i32();
                }
                if tok.check_value("float") {
                    return aty_f64();
                }
                if tok.check_value("str") {
                    return AType::Pri(APriType::Str);
                }

                todo!("ty: {}", tok);
            }
        }
    }

}