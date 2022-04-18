use super::{
    ParseResult2, Parser, SyntaxNode as SN, SyntaxType as ST, TokenTree,
};


impl Parser {
    pub(crate) fn parse_stmt(&mut self) -> ParseResult2 {
        let four = ST::Item;
        let mut subs = vec![];

        if self.peek1_t().check_name("let") {
            subs.push((ST::r#let, box SN::E(self.unchecked_advance())));
            subs.push((ST::PatNoTop, box SN::T(self.parse_pat_no_top()?)));

            // Skip Type

            if self.peek1_t().check_name("assign") {
                subs.push((ST::assign, box SN::E(self.unchecked_advance())))
            }
            subs.push((ST::Expr, box SN::T(self.parse_expr()?)));
        }
        else {
            subs.push((ST::Expr, box SN::T(self.parse_expr()?)));
        }

        subs.push((ST::semi, box SN::E(self.expect_eat_semi_t(four)?)));

        Ok(TokenTree::new(subs))
    }

    pub(crate) fn parse_stmts(&mut self) -> ParseResult2 {
        let mut subs = vec![];

        loop {

            if self.peek1_t().check_name("let") {
                subs.push((ST::Stmt, box SN::T(self.parse_stmt()?)));
            }
            else {
                let expr_sn = box SN::T(self.parse_expr()?);
                if self.peek1_t().check_name("semi") {
                    let semi_sn = box SN::E(self.unchecked_advance());
                    let stmt_tt = TokenTree::new(vec![
                        (ST::Expr, expr_sn),
                        (ST::semi, semi_sn),
                    ]);
                    subs.push((ST::Stmt, box SN::T(stmt_tt)));
                } else {
                    subs.push((ST::Expr, expr_sn));
                    break;
                }
            }

            if self.peek1_t().check_name("rbrace") {
                break;
            }
        }

        Ok(TokenTree::new(subs))
    }
}
