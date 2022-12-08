use m6lexerkit::Token;
use m6parserkit::{parse_infix_expr, BopWrapper, InfixExpr};

use super::{
    ParseErrorReason as R, ParseResult2, Parser, SyntaxNode as SN,
    SyntaxType as ST, TokenTree,
};


impl Parser {
    pub(crate) fn parse_expr(&mut self) -> ParseResult2 {
        let mut expr_units = vec![];
        let mut ops = vec![];

        loop {
            let tok1 = self.peek1_t();
            let ty;
            let tt;

            #[cfg(test)]
            #[allow(unused)]
            {
                let tok1_name = tok1.name_string();
                let tok1_value = tok1.value_string();

                if !self.is_end() {
                    let tok2_name = self.peek2_t().name_string();
                    let tok2_value = self.peek2_t().value_string();
                }
            }

            /* ExprBlk */
            if tok1.check_name("if") {
                ty = ST::IfExpr;
                tt = box SN::T(self.parse_if_expr()?);
            } else if tok1.check_name("loop") {
                ty = ST::InfiLoopExpr;
                tt = box SN::T(self.parse_infi_loop_expr()?);
            } else if tok1.check_name("while") {
                // item = (ST::InfiLoopExpr, box SN::T(self.parse_infi_loop_expr()?))
                todo!()
            } else if tok1.check_name("for") {
                todo!()
            } else if tok1.check_name("lparen") {
                ty = ST::GroupedExpr;
                tt = box SN::T(self.parse_grouped_expr()?);
            }
            /* ExprSpan */
            // LitExpr
            else if tok1.check_name("lit_char") {  // u32 in memorry
                ty = ST::LitExpr;
                tt = box SN::T(TokenTree::new(vec![(
                    ST::lit_char,
                    box SN::E(self.unchecked_advance())
                )]));
            } else if tok1.check_name("lit_str") {  // char*
                ty = ST::LitExpr;
                tt = box SN::T(TokenTree::new(vec![(
                    ST::lit_str,
                    box SN::E(self.unchecked_advance())
                )]));
            } else if tok1.check_name("lit_rawstr") {  // encoded char*
                ty = ST::LitExpr;
                tt = box SN::T(TokenTree::new(vec![(
                    ST::lit_rawstr,
                    box SN::E(self.unchecked_advance())
                )]));
            } else if tok1.check_name("lit_int") {  // i32
                ty = ST::LitExpr;
                tt = box SN::T(TokenTree::new(vec![(
                    ST::lit_int,
                    box SN::E(self.unchecked_advance())
                )]));
            } else if tok1.check_name("lit_float") {  // f64
                ty = ST::LitExpr;
                tt = box SN::T(TokenTree::new(vec![(
                    ST::lit_float,
                    box SN::E(self.unchecked_advance())
                )]));
            } else if tok1.check_name("lit_bool") {  // u8
                ty = ST::LitExpr;
                tt = box SN::T(TokenTree::new(vec![(
                    ST::lit_bool,
                    box SN::E(self.unchecked_advance())
                )]));
            }
            // PathExpr | SideEffectExpr
            else if tok1.check_name("id") {
                let tok2 = self.peek2_t();

                if tok2.check_values_in(&["++", "--"]) {
                    ty = ST::SideEffectExpr;
                    tt = box SN::T(self.parse_side_effect_expr()?);
                }
                else {
                    ty = ST::PathExpr;
                    tt = box SN::T(self.parse_path_expr()?);
                }
            } else if tok1.check_name("continue") {
                // ty = ST::ContinueExpr;
                // tt = box SN::T(self.parse_return_expr()?);
                todo!()
            } else if tok1.check_name("break") {
                // ty = ST::BreakExpr;
                // tt = box SN::T(self.parse_return_expr()?);
                todo!()
            } else if tok1.check_name("return") {
                ty = ST::ReturnExpr;
                tt = box SN::T(self.parse_return_expr()?);
            } else if tok1.check_name("cmd") {
                ty = ST::CmdExpr;
                tt = box SN::T(self.parse_cmd_expr()?);
            }
            // Op
            // Precedence 110
            else if tok1.check_name("as") {
                ops.push(BopWrapper::new(
                    (ST::r#as, self.unchecked_advance()),
                    100,
                ));
                continue;
            }
            // Precedence 100
            else if tok1.check_name("mul") {
                ops.push(BopWrapper::new(
                    (ST::mul, self.unchecked_advance()),
                    100,
                ));
                continue;
            } else if tok1.check_name("div") {
                ops.push(BopWrapper::new(
                    (ST::div, self.unchecked_advance()),
                    100,
                ));
                continue;
            } else if tok1.check_name("percent") {
                ops.push(BopWrapper::new(
                    (ST::percent, self.unchecked_advance()),
                    100,
                ));
                continue;
            }
            // Precedence 90
            else if tok1.check_name("add") {
                ops.push(BopWrapper::new(
                    (ST::add, self.unchecked_advance()),
                    90,
                ));
                continue;
            } else if tok1.check_name("sub") {
                ops.push(BopWrapper::new(
                    (ST::sub, self.unchecked_advance()),
                    90,
                ));
                continue;
            }
            // Precedence 80
            else if tok1.check_name("lshf") {
                ops.push(BopWrapper::new(
                    (ST::lshf, self.unchecked_advance()),
                    80,
                ));
                continue;
            } else if tok1.check_name("rshf") {
                ops.push(BopWrapper::new(
                    (ST::rshf, self.unchecked_advance()),
                    80,
                ));
                continue;
            }
            // Percedence 70
            else if tok1.check_name("band") {
                ops.push(BopWrapper::new(
                    (ST::band, self.unchecked_advance()),
                    70,
                ));
                continue;
            }
            // Percedence 60
            else if tok1.check_name("bxor") {
                ops.push(BopWrapper::new(
                    (ST::bxor, self.unchecked_advance()),
                    60,
                ));
                continue;
            }
            // Percedence 50
            else if tok1.check_name("bor") {
                ops.push(BopWrapper::new(
                    (ST::bor, self.unchecked_advance()),
                    50,
                ));
                continue;
            }
            // Percedence 40
            else if tok1.check_name("eq") {
                ops.push(BopWrapper::new(
                    (ST::eq, self.unchecked_advance()),
                    40,
                ));
                continue;
            } else if tok1.check_name("neq") {
                ops.push(BopWrapper::new(
                    (ST::neq, self.unchecked_advance()),
                    40,
                ));
                continue;
            } else if tok1.check_name("gt") {
                ops.push(BopWrapper::new(
                    (ST::gt, self.unchecked_advance()),
                    40,
                ));
                continue;
            } else if tok1.check_name("ge") {
                ops.push(BopWrapper::new(
                    (ST::ge, self.unchecked_advance()),
                    40,
                ));
                continue;
            } else if tok1.check_name("lt") {
                ops.push(BopWrapper::new(
                    (ST::lt, self.unchecked_advance()),
                    40,
                ));
                continue;
            } else if tok1.check_name("le") {
                ops.push(BopWrapper::new(
                    (ST::le, self.unchecked_advance()),
                    40,
                ));
                continue;
            }
            // Percedence 30
            else if tok1.check_name("and") {
                ops.push(BopWrapper::new(
                    (ST::and, self.unchecked_advance()),
                    30,
                ));
                continue;
            }
            // Percedence 20
            else if tok1.check_name("or") {
                ops.push(BopWrapper::new(
                    (ST::or, self.unchecked_advance()),
                    20,
                ));
                continue;
            }
            // Percedence 10 assign
            else if tok1.check_name("assign") {
                ops.push(BopWrapper::new(
                    (ST::assign, self.unchecked_advance()),
                    10,
                ));
                continue;
            } else if tok1.check_name("add_assign") {
                ops.push(BopWrapper::new(
                    (ST::add_assign, self.unchecked_advance()),
                    10,
                ));
                continue;
            } else if tok1.check_name("sub_assign") {
                ops.push(BopWrapper::new(
                    (ST::sub_assign, self.unchecked_advance()),
                    10,
                ));
                continue;
            } else if tok1.check_name("mul_assign") {
                ops.push(BopWrapper::new(
                    (ST::mul_assign, self.unchecked_advance()),
                    10,
                ));
                continue;
            } else if tok1.check_name("div_assign") {
                ops.push(BopWrapper::new(
                    (ST::div_assign, self.unchecked_advance()),
                    10,
                ));
                continue;
            } else if tok1.check_name("percent_assign") {
                ops.push(BopWrapper::new(
                    (ST::percent_assign, self.unchecked_advance()),
                    10,
                ));
                continue;
            }
            // Percedence 0 assign
            else if tok1.check_name("band_assign") {
                ops.push(BopWrapper::new(
                    (ST::band_assign, self.unchecked_advance()),
                    0,
                ));
                continue;
            } else if tok1.check_name("bor_assign") {
                ops.push(BopWrapper::new(
                    (ST::bor_assign, self.unchecked_advance()),
                    0,
                ));
                continue;
            } else if tok1.check_name("bxor_assign") {
                ops.push(BopWrapper::new(
                    (ST::bxor_assign, self.unchecked_advance()),
                    0,
                ));
                continue;
            } else if tok1.check_name("lshf_assign") {
                ops.push(BopWrapper::new(
                    (ST::lshf_assign, self.unchecked_advance()),
                    0,
                ));
                continue;
            } else if tok1.check_name("rshf_assign") {
                ops.push(BopWrapper::new(
                    (ST::rshf_assign, self.unchecked_advance()),
                    0,
                ));
                continue;
            } else {
                break;
            }

            expr_units.push((ty, tt));
        }

        if expr_units.is_empty() {
            return Err(R::Unrecognized { four: ST::Expr, found: *self.peek1_t() })
        }

        if ops.is_empty() {
            Ok(TokenTree::new(expr_units))
        } else {
            Ok(map_infix_expr_to_tt(parse_infix_expr(ops, expr_units)))
        }

    }

    pub(crate) fn parse_infi_loop_expr(&mut self) -> ParseResult2 {
        let four = ST::InfiLoopExpr;
        let mut subs = vec![];

        self.expect_eat_tok1_t(ST::r#loop, four)?;

        subs.push((ST::BlockExpr, box SN::T(self.parse_block_expr()?)));

        Ok(TokenTree::new(subs))
    }

    pub(crate) fn parse_if_expr(&mut self) -> ParseResult2 {
        let four = ST::IfExpr;
        let mut subs = vec![];
        subs.push((
            ST::r#if,
            box SN::E(self.expect_eat_tok1_t(ST::r#if, four)?))
        );
        subs.push((ST::Expr, box SN::T(self.parse_expr()?)));
        subs.push((ST::BlockExpr, box SN::T(self.parse_block_expr()?)));

        #[cfg(test)]
        {
            let _tok_name = self.peek1_t().name_string();
            let _tok_value = self.peek1_t().value_string();
        }

        if self.peek1_t().check_name("else") {
            self.unchecked_advance();
            let lookhead1 = self.peek1_t();

            if lookhead1.check_name("lbrace") {
                subs.push((
                    ST::BlockExpr,
                    box SN::T(self.parse_block_expr()?),
                ));
            } else if lookhead1.check_name("if") {
                subs.push((ST::IfExpr, box SN::T(self.parse_if_expr()?)));
            } else {
                return Err(R::Unrecognized {
                    four: ST::r#else,
                    found: *lookhead1,
                });
            }
        }

        Ok(TokenTree::new(subs))
    }

    pub(crate) fn parse_block_expr(&mut self) -> ParseResult2 {
        let four = ST::BlockExpr;
        let mut subs = vec![];

        self.expect_eat_tok1_t(ST::lbrace, four)?;
        subs.push((ST::Stmts, box SN::T(self.parse_stmts()?)));
        self.expect_eat_tok1_t(ST::rbrace, four)?;

        Ok(TokenTree::new(subs))
    }

    pub(crate) fn parse_path_expr(&mut self) -> ParseResult2 {
        let four = ST::PathExpr;
        let mut subs = vec![];

        subs.push((ST::PathExprSeg, box SN::T(self.parse_path_expr_seg()?)));

        while self.peek1_t().check_name("colo2") {
            self.expect_eat_tok1_t(ST::colon2, four)?;
            subs.push((
                ST::PathExprSeg,
                box SN::T(self.parse_path_expr_seg()?),
            ));
        }

        Ok(TokenTree::new(subs))
    }

    pub(crate) fn parse_path_expr_seg(&mut self) -> ParseResult2 {
        let four = ST::PathExprSeg;
        let mut subs = vec![];

        let tok = self.expect_eat_id_t(four)?;

        subs.push((ST::id, box SN::E(tok)));

        Ok(TokenTree::new(subs))
    }

    pub(crate) fn parse_grouped_expr(&mut self) -> ParseResult2 {
        let four = ST::LitExpr;
        let mut subs = vec![];

        self.expect_eat_tok1_t(ST::lparen, four)?;
        subs.push((ST::Expr, box SN::T(self.parse_expr()?)));
        self.expect_eat_tok1_t(ST::rparen, four)?;

        Ok(TokenTree::new(subs))
    }

    pub(crate) fn parse_return_expr(&mut self) -> ParseResult2 {
        let four = ST::ReturnExpr;
        let mut subs = vec![];

        self.expect_eat_tok1_t(ST::r#return, four)?;
        if !self.peek1_t().check_names_in(&["semi", "rbrace"]) {
            subs.push((ST::Expr, box SN::T(self.parse_expr()?)));
        }

        Ok(TokenTree::new(subs))
    }


    pub(crate) fn parse_cmd_expr(&mut self) -> ParseResult2 {
        let four = ST::CmdExpr;
        let mut subs = vec![];

        let cmd_tok = self.expect_eat_tok1_t(ST::cmd, four)?;

        subs.push((ST::cmd, box SN::E(cmd_tok)));

        Ok(TokenTree::new(subs))
    }

    pub(crate) fn parse_side_effect_expr(&mut self) -> ParseResult2 {
        let four = ST::SideEffectExpr;

        let tok1 = self.peek1_t();
        let mut subs = vec![];

        if tok1.check_name("id") {
            subs.push((ST::id, box SN::E(self.unchecked_advance())));

            if self.peek1_t().check_name("inc") {
                subs.push((ST::inc, box SN::E(self.unchecked_advance())));
            } else if self.peek1_t().check_name("dec") {
                subs.push((ST::dec, box SN::E(self.unchecked_advance())));
            } else {
                return Err(R::Unrecognized {
                    four,
                    found: self.unchecked_advance(),
                });
            }
        } else if tok1.check_name("inc") {
            subs.push((ST::inc, box SN::E(self.unchecked_advance())));
            subs.push((ST::id, box SN::E(self.expect_eat_id_t(four)?)));
        } else if tok1.check_name("dec") {
            subs.push((ST::dec, box SN::E(self.unchecked_advance())));
            subs.push((ST::id, box SN::E(self.expect_eat_id_t(four)?)));
        }

        Ok(TokenTree::new(subs))
    }
}

fn map_infix_expr_to_tt(
    ie: InfixExpr<BopWrapper<(ST, Token)>, (ST, Box<SN>)>,
) -> TokenTree {
    let mut subs = vec![];

    match ie {
        InfixExpr::E(e) => subs.push(e),
        InfixExpr::T { bop, pri1, pri2 } => {
            subs.push((ST::OpExpr, box SN::T(map_infix_expr_to_tt(*pri1))));
            let (bopty, boptt) = bop.unwrap();
            subs.push((bopty, box SN::E(boptt)));
            subs.push((ST::OpExpr, box SN::T(map_infix_expr_to_tt(*pri2))));
        }
    }

    TokenTree::new(subs)
}
