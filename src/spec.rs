use m6lexerkit::lazy_static::lazy_static;
use m6parserkit::*;


lazy_static! {
    pub static ref GRAMMER: BNF = gen_bnf(SPEC);
}


/// 重复匹配按照优先顺序
const SPEC: &str = r#"
Module:
  | [Item]*

Item:
  | [Attrs]? [Function]
#  | [CupBoard]

# 函数定义或外部函数声明
Function:
  | <fn> <id> <lparen> [FnParams]? <rparen> (<rarrow> [Type])?
    ([BlockExpr] | <semi>)

FnParams:
  | [FnParam]? (<comma> [FnParam])*

FnParam:
  | ([PatNoTop] <colon> [Type] | [Type])

PatNoTop:
  | <id>

Type:
  | <id>
  | <lbracket> <id> <rbracket>

Attrs:
  | <attr>*

Stmts:
  | [Stmt]* [Expr]?

Stmt:
  | <let> [PatNoTop] (<colon> [Type])? (<assign> [Expr])? <semi>
  | [Expr] <semi>

Expr:
  | [Expr] <add> [Expr]
  | [Expr] <sub> [Expr]
  | [Expr] <mul> [Expr]
  | [Expr] <div> [Expr]
  | [Expr] <percent> [Expr]
  | [Expr] <lshf> [Expr]
  | [Expr] <rshf> [Expr]
  | [Expr] <eq> [Expr]
  | [Expr] <neq> [Expr]
  | [Expr] <gt> [Expr]
  | [Expr] <lt> [Expr]
  | [Expr] <ge> [Expr]
  | [Expr] <le> [Expr]
  | [Expr] <assign> [Expr]
  | [Expr] <add_assign> [Expr]
  | [Expr] <sub_assign> [Expr]
  | [Expr] <mul_assign> [Expr]
  | [Expr] <div_assign> [Expr]
  | [Expr] <percent_assign> [Expr]
  | [Expr] <lshf_assign> [Expr]
  | [Expr] <rshf_assign> [Expr]
  | [Expr] <or> [Expr]
  | [Expr] <and> [Expr]
  | [Expr] <as> [Expr]
  | [IfExpr]
  | [InfiLoopExpr]
  | [GroupedExpr]
  | [BlockExpr]
  | [LitExpr]
  | [SideEffectExpr]
  | [PathExpr]
  | [ReturnExpr]
  | [CmdExpr]
  | [FunCallExpr]
  | [Expr]

BlockExpr:
  | <lbrace> [Stmts] <rbrace>

IfExpr:
  | <if> [Expr] [BlockExpr] (<else> ([BlockExpr] | [IfExpr]))?

InfiLoopExpr:
  | <loop> [BlockExpr]

GroupedExpr:
  | <lparen> [Expr] <rparen>

LitExpr:
  | <lit_char>
  | <lit_str>
  | <lit_rawstr>
  | <lit_int>
  | <lit_float>
  | <lit_bool>

SideEffectExpr:
  | <inc> <id>
  | <dec> <id>
  | <id> <inc>
  | <id> <dec>

PathExpr:
  | <tag>? [PathExprSeg](<colon2> [PathExprSeg])*

# 未来可能包含泛型参数之类
PathExprSeg:
  | <id>

ReturnExpr:
  | <ret> [Expr]?

CmdExpr:
  | <cmd>

FunCallExpr:
  | [PathExpr] [GroupedExpr]

"#;


#[cfg(test)]
mod tests {
    use m6lexerkit::SrcFileInfo;

    use super::*;
    use crate::{lexer::tokenize, parser::parse};

    #[test]
    fn test_spec_gen() {
        let bnf = gen_bnf(SPEC);

        println!("{bnf:#?}");
    }

    #[test]
    fn test_spec_verify() {
        let src = SrcFileInfo::new(&"examples/test_spec.bath").unwrap();

        let tokens = tokenize(&src).unwrap();

        let tt = parse(tokens, &src).unwrap();

        // println!("tt: {tt:#?}");
        crate::spec::GRAMMER.verify(&tt).unwrap();
    }
}
