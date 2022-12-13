
## 总概

编译型的bash增强的语言


## 语法结构

```
Module:
  | [Item]*

Item:
  | [Function]
  | [CupBoard]

Attrs:
  | <attr>+

Function:  # 函数定义或外部函数声明
  | [Attrs]? <fn> <id> <lparen> [FnParams]? <rparen> (<rarrow> [Type])? ([BlockExpr] | <semi>)

FnParams:
  | [FnParam]? (<comma> [FnParam])*

FnParam:
  | ([PatNoTop] <colon> [Type] | [Type])

PatNoTop:
  | [IdentPat]

IdentPat:
  | <id>

BlockExpr:
  | <lbrace> [Stmts] <rbrace>

Stmts:
  | [Stmt]+
  | [Stmt]* [ExprBlk]

Stmt:
  | [Item]
  | [LetStmt]
  | [ExprStmt]

LetStmt:
  | <let> [PatNoTop] (<semi> [Type])? (<assign> [Expr])? <semi>

ExprStmt:
  | [ExprSpan] <semi>
  | [ExprBlk] <semi>?

Expr:
  | [ExprSpan]
  | [ExprBlk]

ExprSpan:
  | [LitExpr]
  | [PathExpr]
  | [OpExpr]
  | [GroupedExpr]
  | [ReturnExpr]
  | [CmdExpr]
  | [SideEffectExpr]
  | [FunCallExpr]

FunCallExpr:
  | [PathExpr] [GroupedExpr]

ExprBlk:
  | [BlockExpr]
  | [IfExpr]
  | [LoopExpr]

LitExpr:
  | <lit_char>
  | <lit_str>
  | <lit_rawstr>
  | <lit_int>
  | <lit_float>
  | <lit_bool>

PathExpr:
  | [PathExprSeg](<colon2> [PathExprSeg])*

PathExprSeg:
  | <id>

OpExpr:
  | [A.L.Expr]  // Arithmetic or Logical Expression
  | [ComparisonExpr]
  | [LazyBooleanExpr]
  | [TypeCastExpr]
  | [AssignExpr]
  | [CompAssignExpr]

A.L.Expr:
  | [Expr] <add> [Expr]
  | [Expr] <sub> [Expr]
  | [Expr] <mul> [Expr]
  | [Expr] <div> [Expr]
  | [Expr] <percent> [Expr]
  | [Expr] <lshf> [Expr]
  | [Expr] <rshf> [Expr]

ComparisonExpr:
  | [Expr] <eq> [Expr]
  | [Expr] <neq> [Expr]
  | [Expr] <gt> [Expr]
  | [Expr] <lt> [Expr]
  | [Expr] <ge> [Expr]
  | [Expr] <le> [Expr]

LazyBooleanExpr:
  | [Expr] <or> [Expr]
  | [Expr] <and> [Expr]

TypeCastExpr:
  | [Expr] <as> [Type]

AssignExpr:
  | [PathExpr] <assign> [Expr]

CompAssignExpr:
  | [Expr] <add_assign> [Expr]
  | [Expr] <sub_assign> [Expr]
  | [Expr] <mul_assign> [Expr]
  | [Expr] <div_assign> [Expr]
  | [Expr] <percent_assign> [Expr]
  | [Expr] <lshf_assign> [Expr]
  | [Expr] <rshf_assign> [Expr]

CmdExpr:
  | <cmd>

SideEffectExpr:
  | <inc> <id>
  | <dec> <id>
  | <id> <inc>
  | <id> <dec>

Type:
  | <id>
  | lbracket <id> <rbracket>


GroupedExpr:
  | <lparen> [Expr] <rparen>

ReturnExpr:
  | <return> [Expr]?

IfExpr:
  | <if> [ExprSpan] [BlockExpr] (<else> ([BlockExpr] | [IfExpr])?)?

LoopExpr:
  | [InfiLoopExpr]

InfiLoopExpr:
  | <loop> [BlockExpr]


```

<!-- ### 循环
loop {

}

while *COND* {

}

for p in range {

}

break / return / continue

### 条件判断

if COND else
if COND elif elif else -->




## 数据结构(builtin)

### Dynamic Array: [T]

[i32]/[u8]/[ptr]


## 属性注解(attr)


**no_mangle:**

  1. 函数 fullname = basename,
  2. 暗示了 `unique`, 同名函数只能有一个，没有依据类型的分发（dispatch）
  3. 允许变长参数

**symbol_name**

  `@sym(xxx)`
