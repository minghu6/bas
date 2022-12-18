use super::{
    ParseErrorReason as R, ParseResult2, Parser, SyntaxNode as SN,
    SyntaxType as ST, TT,
};

impl Parser {
    pub(crate) fn parse_item(&mut self) -> ParseResult2 {
        let four = ST::Item;

        let mut subs = vec![];

        if self.peek1_t().check_name("attr") {
            subs.push((ST::Attrs, SN::T(self.parse_attrs()?)));
        }

        if self.peek1_t().check_name("fn") {
            subs.push((ST::Function, SN::T(self.parse_fn()?)));
            return Ok(TT::new(subs));
        } else if !subs.is_empty() {
            return Err(R::Expect {
                expect: four,
                four: ST::Attrs,
                found: self.unchecked_advance(),
            });
        }

        Err(R::Unrecognized {
            four,
            found: self.unchecked_advance(),
        })
    }


    pub(crate) fn parse_fn(&mut self) -> ParseResult2 {
        let four = ST::Function;
        let mut subs = vec![];

        subs.push((
            ST::r#fn,
            SN::E(self.expect_eat_tok1_t(ST::r#fn, four)?)
        ));

        subs.push((ST::id, SN::E(self.expect_eat_id_t(four)?))); // function name

        if self.peek1_t().check_name("lparen") {
            subs.push((ST::lparen, SN::E(self.unchecked_advance())));
        } else {
            return Err(R::Expect {
                expect: ST::lparen,
                four: ST::Function,
                found: *self.peek1_t(),
            });
        }

        subs.push((ST::FnParams, SN::T(self.parse_fn_params()?)));
        subs.push((
            ST::rparen,
            SN::E(self.expect_eat_tok1_t(ST::rparen, four)?),
        ));

        if self.peek1_t().check_name("rarrow") {
            subs.push((ST::rarrow, SN::E(self.unchecked_advance())));

            let ret = SN::T(self.parse_ty()?);
            subs.push((ST::Type, ret));
        }

        if self.peek1_t().check_name("semi") {
            subs.push((ST::semi, SN::E(self.unchecked_advance())));
        } else {
            subs.push((ST::BlockExpr, SN::T(self.parse_block_expr()?)));
        }

        Ok(TT::new(subs))
    }


    fn parse_fn_params(&mut self) -> ParseResult2 {
        let four = ST::FnParams;
        let mut subs = vec![];

        if !self.peek1_t().check_name("rparen") {
            loop {
                let fn_param = SN::T(self.parse_fn_param()?);
                subs.push((ST::FnParam, fn_param));

                if self.peek1_t().check_name("rparen") {
                    break;
                }

                // eat comma
                subs.push((
                    ST::comma,
                    SN::E(self.expect_eat_tok1_t(ST::comma, four)?),
                ));
            }
        }

        return Ok(TT::new(subs));
    }


    fn parse_fn_param(&mut self) -> ParseResult2 {
        let mut subs = vec![];
        let _four = ST::FnParam;

        // Check If the FnParam is followed by [PatNoTop] or just [Type]
        if self.peek1_t().check_value("[")
            || !self.peek2_t().check_name("colon")
        {
            subs.push((ST::Type, SN::T(self.parse_ty()?)))
        } else {
            subs.push((ST::PatNoTop, SN::T(self.parse_pat_no_top()?)));
            subs.push((
                ST::colon,
                SN::E(self.expect_eat_colon_t(ST::FnParamPat)?),
            ));
            subs.push((ST::Type, SN::T(self.parse_ty()?)));
        }

        return Ok(TT::new(subs));
    }
}
