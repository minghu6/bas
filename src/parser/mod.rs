use std::path::{Path, PathBuf};

use m6lexerkit::{SrcFileInfo, SrcLoc, Token};
use m6parserkit::gen_syntax_enum;

mod item;
mod pat;
mod ty;
mod expr;
mod stmt;

gen_syntax_enum! [ pub SyntaxType |
    Item,
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
    OpExpr,
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

    r#fn,
    r#let,
    id,
    ret,
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
    r#return,
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
    eof
];

pub(crate) struct TokenTree {
    pub(crate) subs: Vec<(SyntaxType, Box<SyntaxNode>)>,
}

impl TokenTree {
    pub(super) fn new(subs: Vec<(SyntaxType, Box<SyntaxNode>)>) -> Self {
        Self { subs }
    }
}

impl std::fmt::Debug for TokenTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let mut dbs = &mut f.debug_tuple("=>");

        for (ty, sn) in self.subs.iter() {
            dbs = dbs.field(ty);

            match &**sn {
                SyntaxNode::T(tt) => dbs = dbs.field(&tt),
                SyntaxNode::E(tok) => dbs = dbs.field(&tok),
            }
        }

        dbs.finish()
    }
}

#[derive(Debug)]
pub(crate) enum SyntaxNode {
    T(TokenTree),
    E(Token),
}

impl SyntaxNode {
    // pub(super) fn from_t(subs: Vec<(SyntaxType, Box<SyntaxNode>)>) -> Self {
    //     Self::T(TokenTree { subs })
    // }
}


pub struct Parser {
    cursor: usize,
    tokens: Vec<Token>,
    eof: Token
}

#[allow(unused)]
#[derive(Debug)]
pub struct ParseError {
    reason: ParseErrorReason,
    loc: SrcLoc,
    path: PathBuf,
}
impl std::error::Error for ParseError {}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
pub(crate) type ParseResult = Result<TokenTree, ParseError>;
pub(crate) type ParseResult2 = Result<TokenTree, ParseErrorReason>;


#[derive(Debug)]
pub enum ParseErrorReason {
    Expect {
        expect: SyntaxType,
        four: SyntaxType,
        found: Token,
    },
    Unrecognized {
        four: SyntaxType,
        found: Token
    }
}

impl ParseErrorReason {
    pub(super) fn emit_error(self, loc: SrcLoc, path: &Path) -> ParseError {
        match self {
            _ => ParseError {
                reason: self,
                loc,
                path: path.to_owned(),
            },
        }
    }
}

pub(crate) fn parse(tokens: Vec<Token>, srcfile: &SrcFileInfo) -> ParseResult {
    let mut parser = Parser::new(tokens);

    parser.parse(srcfile)
}



impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { cursor: 0, tokens, eof: Token::eof() }
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

    fn last_t(&self) -> &Token {
        if self.cursor == 0 {
            panic!("cursor at ZEOR hasn't last");
        }

        &self.tokens[self.cursor - 1]
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

    fn is_end(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    fn parse_(&mut self) -> ParseResult2 {
        let mut subs = vec![];

        while !self.is_end() {
            subs.push((
                SyntaxType::Item,
                box SyntaxNode::T(self.parse_item()?),
            ))
        }

        Ok(TokenTree { subs })
    }

    fn parse(&mut self, srcfile: &SrcFileInfo) -> ParseResult {
        self.parse_().or_else(|reason| {
            Err(reason.emit_error(
                srcfile.boffset2srcloc(self.last_t().span().from),
                srcfile.get_path(),
            ))
        })
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

    use crate::lexer::tokenize;
    use super::parse;

    #[test]
    fn test_parser() -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from("./examples/exp0.bath");
        let src = SrcFileInfo::new(&path).unwrap();

        // println!("{:#?}", sp_m(srcfile.get_srcstr(), SrcLoc { ln: 0, col: 0 }));

        let tokens = tokenize(&src)?;
        let tt = parse(tokens, &src)?;

        println!("{:#?}", tt);

        Ok(())
    }
}
