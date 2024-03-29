#![allow(unused_imports)]

use m6lexerkit::Token;

use super::{
    ParseErrorReason as R, ParseResult2, Parser, SyntaxNode as SN,
    SyntaxType as ST, TT,
};


impl Parser {
    pub(crate) fn parse_ty(&mut self) -> ParseResult2 {
        let four = ST::Type;
        let mut subs = vec![];

        if self.peek1_t().check_value("[") {
            subs.push((ST::lbracket, SN::E(self.expect_eat_tok1_t(ST::lbracket, four)?)));
            subs.push((ST::id, SN::E(self.expect_eat_id_t(four)?)));
            subs.push((ST::rbracket, SN::E(self.expect_eat_tok1_t(ST::rbracket, four)?)));
        }
        else {
            subs.push((ST::id, SN::E(self.expect_eat_id_t(four)?)));
        }

        return Ok(TT::new(subs));
    }
}
