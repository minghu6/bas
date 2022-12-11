use super::{
    ParseResult2, Parser, SyntaxNode as SN, SyntaxType as ST, TokenTree,
};




impl Parser {
    pub(crate) fn parse_attrs(&mut self) -> ParseResult2 {
        let mut subs = vec![];

        while self.peek1_t().check_name("attr") {
            subs.push((ST::attr, SN::E(self.unchecked_advance())));
        }

        Ok(TokenTree { subs })
    }
}
