#![allow(unused_imports)]

use m6lexerkit::Token;

use super::{
    ParseErrorReason as R, ParseResult2, Parser, SyntaxNode as SN,
    SyntaxType as ST, TT,
};


impl Parser {
    pub(crate) fn parse_pat_no_top(&mut self) -> ParseResult2 {
        let four = ST::PatNoTop;
        let mut subs = vec![];

        subs.push((ST::id, SN::E(self.expect_eat_id_t(four)?)));

        return Ok(TT::new(subs));
    }
}
