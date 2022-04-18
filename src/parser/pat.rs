#![allow(unused_imports)]

use m6lexerkit::Token;

use super::{
    ParseErrorReason as R, ParseResult2, Parser, SyntaxNode as SN,
    SyntaxType as ST, TokenTree,
};


impl Parser {
    pub(crate) fn parse_pat_no_top(&mut self) -> ParseResult2 {
        let four = ST::Type;
        let mut subs = vec![];

        subs.push((ST::id, box SN::E(self.expect_eat_id_t(four)?)));

        return Ok(TokenTree::new(subs));
    }
}
