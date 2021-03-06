use m6lexerkit::Token;

use super::{
    ParseErrorReason as R, ParseResult2, Parser, SyntaxNode as SN,
    SyntaxType as ST, TokenTree,
};

impl Parser {
    pub(crate) fn parse_item(&mut self) -> ParseResult2 {
        let four = ST::Item;
        let tok1 = self.peek1_t();

        if tok1.check_name("fn") {
            return Ok(TokenTree::new(vec![(
                ST::Function,
                box SN::T(self.parse_fn()?),
            )]));
        }

        Err(R::Unrecognized { four, found: *tok1 })
    }

    pub(crate) fn parse_fn(&mut self) -> ParseResult2 {
        let four = ST::Function;

        self.expect_eat_tok1_t(ST::r#fn, four)?;
        let mut subs = vec![];

        subs.push((ST::id, box SN::E(self.expect_eat_id_t(four)?)));  // function name

        if self.peek1_t().check_name("lparen") {
            self.unchecked_advance();
        }
        else {
            return Err(R::Expect {
                expect: ST::lparen,
                four: ST::Function,
                found: *self.peek1_t(),
            });
        }

        if !self.peek1_t().check_name("rparen") {
            subs.push((ST::FnParams, box SN::T(self.parse_fn_params()?)));
        }

        self.expect_eat_tok1_t(ST::rparen, four)?;

        if self.peek1_t().check_name("rarrow") {
            self.unchecked_advance();

            let ret = box SN::T(self.parse_ty()?);
            subs.push((ST::ret, ret));
        }

        subs.push((ST::BlockExpr, box SN::T(self.parse_block_expr()?)));

        Ok(TokenTree::new(subs))
    }


    fn parse_fn_params(&mut self) -> ParseResult2 {
        let mut subs = vec![];

        loop {
            let fn_param = box SN::T(self.parse_fn_param()?);
            subs.push((ST::FnParam, fn_param));

            if self.peek1_t().check_name("rparen") {
                break;
            }

            self.unchecked_advance();  // eat comma
        }

        return Ok(TokenTree::new(subs));
    }

    fn parse_fn_param(&mut self) -> ParseResult2 {
        let mut subs = vec![];
        let four = ST::FnParam;

        // Check If the FnParam is followed by [PatNoTop] or just [Type]
        let end = self.tokens.len();
        for i in self.cursor..end {
            if self.tokens[i].check_name("id") {
                if i + 1 >= end {
                    return Err(
                        R::Expect { expect: ST::FnParam, four, found: Token::eof() }
                    );
                }
                // Differ Function Declaration and Function Definition.
                if self.tokens[i+1].check_name("colon") {
                    subs.push((
                        ST::FnParamPat,
                        box SN::T(self.parse_fn_param_pat()?)
                    ))
                }
                else {
                    subs.push((
                        ST::Type,
                        box SN::T(self.parse_ty()?)
                    ))
                }
            }
        }

        return Ok(TokenTree::new(subs));
    }

    fn parse_fn_param_pat(&mut self) -> ParseResult2 {
        let mut subs = vec![];

        subs.push((ST::PatNoTop, box SN::T(self.parse_pat_no_top()?)));
        self.expect_eat_colon_t(ST::FnParamPat)?;
        subs.push((ST::Type, box SN::T(self.parse_ty()?)));

        return Ok(TokenTree::new(subs));
    }
}
