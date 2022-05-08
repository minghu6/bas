use crate::ast_lowering::{AType, APriType};
use crate::parser::SyntaxNode as SN;

use super::SemanticAnalyzerPass2;



impl SemanticAnalyzerPass2 {

    pub(crate) fn analyze_ty(&mut self, sn: &SN) -> AType {
        match sn {
            SN::T(_) => todo!(),
            SN::E(tok) => {
                // analyze alias -- skip (inner multiple scan)
                if tok.check_name("int") {
                    return AType::Pri(APriType::Int);
                }
                if tok.check_name("float") {
                    return AType::Pri(APriType::F64);
                }
                if tok.check_name("str") {
                    return AType::Pri(APriType::Str);
                }

                todo!("ty: {}", tok);
            }
        }
    }

}