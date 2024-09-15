use std::fmt::Debug;

use m6lexerkit::{Span, SrcFileInfo, Token};
use m6parserkit::{ gen_syntax_enum, SyntaxNode, TokenTree };

use crate::ref_source;

mod attr;
mod expr;
mod item;
mod pat;
mod stmt;
mod ty;


gen_syntax_enum! [ pub SyntaxType |
    Item,
    Attrs,
    CupBoard,
    Function,
    BlockExpr,
    FnParams,
    FnParam,
    FnParamPat,
    Stmts,
    Stmt,
    Type,
    PatNoTop,
    IdentPat,
    LetStmt,
    ExprStmt,
    Expr,
    ExprSpan,
    ExprBlk,
    LitExpr,
    PathExpr,
    PathExprSeg,
    BOpExpr,
    A_L_Expr,
    ComparisionExpr,
    LazyBooleanExpr,
    TypeCastExpr,
    AssignExpr,
    CompAssignExpr,
    CmdExpr,
    SideEffectExpr,
    GroupedExpr,
    ReturnExpr,
    ContinueExpr,
    BreakExpr,
    IfExpr,
    LoopExpr,
    InfiLoopExpr,
    FunCallExpr,

    r#fn,
    r#let,
    id,
    ret,
    rarrow,

    lparen,
    rparen,
    lbrace,
    rbrace,
    lbracket,
    rbracket,

    comma,
    colon,
    colon2,
    semi,

    r#loop,
    r#if,
    r#else,
    r#continue,
    r#break,
    lit_char,
    lit_str,
    lit_rawstr,
    lit_int,
    lit_float,
    lit_bool,
    cmd,
    inc,
    dec,
    add,
    sub,
    mul,
    div,
    percent,
    r#as,
    eq,
    neq,
    gt,
    ge,
    lt,
    le,
    and,
    or,
    lshf,
    rshf,
    band,
    bor,
    bxor,
    assign,
    add_assign,
    sub_assign,
    mul_assign,
    div_assign,
    percent_assign,
    band_assign,
    bor_assign,
    bxor_assign,
    lshf_assign,
    rshf_assign,
    attr,
    tag,
    eof
];
pub use SyntaxType as ST;

pub type TT = TokenTree<ST>;
pub type SN = SyntaxNode<ST>;



pub struct Parser {
    cursor: usize,
    tokens: Vec<Token>,

    ent_if_cond: bool,
    eof: Token,
}


pub struct ParseError {
    reason: ParseErrorReason,
    src: SrcFileInfo,
}


pub(crate) type ParseResult = Result<TT, ParseError>;
pub(crate) type ParseResult2 = Result<TT, ParseErrorReason>;


#[allow(unused)]
pub enum ParseErrorReason {
    Expect {
        expect: SyntaxType,
        four: SyntaxType,
        found: Token,
    },
    Unrecognized {
        four: SyntaxType,
        found: Token,
    },
    Lack {
        lack: String,
        four: String,
        span: Span,
    },
}


impl Default for ST {
    fn default() -> Self {
        Self::semi
    }
}


impl std::error::Error for ParseError {}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(f)?;

        writeln!(f, "{:?}", self.reason)?;

        let span = self.reason.span();

        ref_source!(span, "^", f, self.src);

        Ok(())
    }
}


impl ParseErrorReason {
    pub(super) fn emit_error(self, src: SrcFileInfo) -> ParseError {
        match self {
            _ => ParseError { reason: self, src },
        }
    }

    pub(super) fn span(&self) -> Span {
        match self {
            Self::Expect {
                expect: _,
                four: _,
                found,
            } => found.span,
            Self::Unrecognized { four: _, found } => found.span,
            Self::Lack {
                lack: _,
                four: _,
                span,
            } => *span,
        }
    }

    pub(crate) fn lack(lack: &str, four: &str, span: Span) -> Self {
        Self::Lack {
            lack: lack.to_string(),
            four: four.to_string(),
            span,
        }
    }
}

impl Debug for ParseErrorReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expect {
                expect,
                four,
                found,
            } => {
                writeln!(f, "Expect an {expect} when parsing {four}, however found {found:?}")
            }
            Self::Unrecognized { four, found } => {
                writeln!(f, "Unrecognized token {found:?} when parsing {four}")
            }
            Self::Lack { lack, four, .. } => {
                writeln!(f, "Lack {lack} for {four} ")
            }
        }
    }
}


pub(crate) fn parse(tokens: Vec<Token>, srcfile: &SrcFileInfo) -> ParseResult {
    let mut parser = Parser::new(tokens);

    let tt = parser.parse(srcfile)?;

    // println!("tt: {tt:#?}");

    crate::spec::GRAMMER.verify(&tt).unwrap();

    Ok(tt)
}


impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            cursor: 0,
            tokens,
            ent_if_cond: false,
            eof: Token::eof(),
        }
    }

    fn unchecked_advance(&mut self) -> Token {
        let tok = self.tokens[self.cursor];
        self.cursor += 1;

        tok
    }

    fn advance(
        &mut self,
        expect: SyntaxType,
        four: SyntaxType,
    ) -> Result<Token, ParseErrorReason> {
        if self.cursor >= self.tokens.len() {
            return Err(ParseErrorReason::Expect {
                expect,
                four,
                found: Token::eof(),
            });
        }

        Ok(self.unchecked_advance())
    }

    fn peek1_t(&self) -> &Token {
        self.peek_t_(0)
    }

    fn peek2_t(&self) -> &Token {
        self.peek_t_(1)
    }

    fn peek_t_(&self, offset: usize) -> &Token {
        let detected_cursor = self.cursor + offset;

        if detected_cursor >= self.tokens.len() {
            return &self.eof;
            // panic!("cursor has reached end");
        }

        &self.tokens[detected_cursor]
    }

    /// Unchecked !
    fn prev_t(&self) -> &Token {
        &self.tokens[self.cursor - 1]
    }

    fn is_end(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    fn parse_(&mut self) -> ParseResult2 {
        let mut subs = vec![];

        while !self.is_end() {
            subs.push((SyntaxType::Item, SyntaxNode::T(self.parse_item()?)))
        }

        Ok(TT { subs })
    }

    fn parse(&mut self, srcfile: &SrcFileInfo) -> ParseResult {
        self.parse_()
            .or_else(|reason|
                Err(reason.emit_error(srcfile.clone()))
            )
            .and_then(|tt| Ok(tt))
    }

    fn expect_eat_id_t(
        &mut self,
        four: SyntaxType,
    ) -> Result<Token, ParseErrorReason> {
        self.expect_eat_tok1_t(SyntaxType::id, four)
    }

    #[allow(unused)]
    fn expect_eat_comma_t(
        &mut self,
        four: SyntaxType,
    ) -> Result<Token, ParseErrorReason> {
        self.expect_eat_tok1_t(SyntaxType::comma, four)
    }

    fn expect_eat_colon_t(
        &mut self,
        four: SyntaxType,
    ) -> Result<Token, ParseErrorReason> {
        self.expect_eat_tok1_t(SyntaxType::colon, four)
    }

    fn expect_eat_semi_t(
        &mut self,
        four: SyntaxType,
    ) -> Result<Token, ParseErrorReason> {
        self.expect_eat_tok1_t(SyntaxType::semi, four)
    }

    fn expect_eat_tok1_t(
        &mut self,
        expect: SyntaxType,
        four: SyntaxType,
    ) -> Result<Token, ParseErrorReason> {
        let tok = self.advance(expect, four)?;

        if !tok.check_name(&expect.name()) {
            Err(ParseErrorReason::Expect {
                expect,
                four,
                found: tok,
            })
        } else {
            Ok(tok)
        }
    }
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use m6lexerkit::SrcFileInfo;

    use super::parse;
    use crate::lexer::tokenize;

    #[test]
    fn test_parser() -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from("./examples/exp1.bath");
        let src = SrcFileInfo::new(&path).unwrap();

        let tokens = tokenize(&src)?;
        let tt = parse(tokens, &src)?;

        println!("{:#?}", tt);

        Ok(())
    }
}
