use m6lexerkit::{Token, Span};
use m6parserkit::{parse_infix_expr, BopWrapper, InfixExpr};

use super::{
    ParseErrorReason as R, ParseResult2, Parser, SN, ParseErrorReason,
    ST, TT,
};


impl Parser {

    pub(crate) fn parse_expr(&mut self) -> ParseResult2 {
        let mut expr_units = vec![];
        let mut ops = vec![];
        let from = self.peek1_t().span().from;

        loop {
            let ty;
            let tt;

            #[cfg(test)]
            #[allow(unused)]
            {
                let tok1 = self.peek1_t().clone();
                let tok1_name = tok1.name_string();
                let tok1_value = tok1.value_string();

                if !self.is_end() {
                    let tok2_name = self.peek2_t().name_string();
                    let tok2_value = self.peek2_t().value_string();
                }
            }

            /* ExprBlk */

            if
            self.ent_if_cond
            && self.peek1_t().check_name("lbrace")
            && expr_units.len() == ops.len() + 1
            {
                break;
            }

            if let Some((_ty, _tt)) = self.try_parse_expr_block()? {
                ty = _ty;
                tt = _tt;
            }

            /* ExprSpan */

            else if let Some((_ty, _tt)) = self.try_parse_expr_span()? {
                ty = _ty;
                tt = _tt;
            }

            /* Bop */

            else if let Some(bop) = self.try_parse_bop() {
                ops.push(bop);
                continue;
            }
            else {
                break;
            }

            if let Some((st, sn)) = expr_units.pop() {
                if st == ST::PathExpr && ty == ST::GroupedExpr {
                    expr_units.push((
                        ST::FunCallExpr,
                        SN::T(TT { subs: vec![
                                (st, sn),
                                (ty, tt)
                            ]
                        })
                    ));
                }
                else {
                    expr_units.push((st, sn));
                    expr_units.push((ty, tt));
                }
            }
            else {
                expr_units.push((ty, tt));
            }

            if expr_units.len() > ops.len() + 1 {
                println!("expr units: {:#?}", expr_units);

                let span = Span {
                    from,
                    end: self.prev_t().span().end,
                };

                return Err(R::lack("Binary Operator", "MultiExpr", span));
            }
        }

        if expr_units.is_empty() {
            return Err(R::Unrecognized { four: ST::Expr, found: *self.peek1_t() })
        }

        if ops.is_empty() {
            Ok(TT::new(expr_units))
        } else {
            if ops.len() + 1 != expr_units.len() {
                println!("Unmathced ops {} - expr_units {}", ops.len(), expr_units.len());
                println!("{ops:#?}");
                println!("{expr_units:#?}");
            }

            Ok(map_infix_expr_to_tt(parse_infix_expr(ops, expr_units)))
        }

    }

    pub(crate) fn parse_infi_loop_expr(&mut self) -> ParseResult2 {
        let four = ST::InfiLoopExpr;
        let mut subs = vec![];

        subs.push((
            ST::r#loop,
            SN::E(self.expect_eat_tok1_t(ST::r#loop, four)?)
        ));

        subs.push((ST::BlockExpr, SN::T(self.parse_block_expr()?)));

        Ok(TT::new(subs))
    }

    pub(crate) fn parse_if_expr(&mut self) -> ParseResult2 {
        let four = ST::IfExpr;
        let mut subs = vec![];

        self.ent_if_cond = true;

        subs.push((
            ST::r#if,
            SN::E(self.expect_eat_tok1_t(ST::r#if, four)?))
        );
        subs.push((ST::Expr, SN::T(self.parse_expr()?)));
        subs.push((ST::BlockExpr, SN::T(self.parse_block_expr()?)));

        self.ent_if_cond = false;

        #[cfg(test)]
        {
            let _tok_name = self.peek1_t().name_string();
            let _tok_value = self.peek1_t().value_string();
        }

        if self.peek1_t().check_name("else") {
            subs.push((
                ST::r#else,
                SN::E(self.unchecked_advance()),
            ));

            if self.peek1_t().check_name("lbrace") {
                subs.push((
                    ST::BlockExpr,
                    SN::T(self.parse_block_expr()?),
                ));
            } else if self.peek1_t().check_name("if") {
                subs.push((ST::IfExpr, SN::T(self.parse_if_expr()?)));
            } else {
                return Err(R::Unrecognized {
                    four: ST::r#else,
                    found: self.unchecked_advance(),
                });
            }
        }

        Ok(TT::new(subs))
    }

    pub(crate) fn parse_block_expr(&mut self) -> ParseResult2 {
        let four = ST::BlockExpr;
        let mut subs = vec![];

        subs.push(
            (ST::lbrace, SN::E(self.expect_eat_tok1_t(ST::lbrace, four)?))
        );
        subs.push(
            (ST::Stmts, SN::T(self.parse_stmts()?))
        );
        subs.push(
            (ST::rbrace, SN::E(self.expect_eat_tok1_t(ST::rbrace, four)?))
        );

        Ok(TT::new(subs))
    }

    pub(crate) fn parse_path_expr(&mut self) -> ParseResult2 {
        let four = ST::PathExpr;
        let mut subs = vec![];

        if self.peek1_t().check_name("tag") {
            // println!("tag: {}", self.peek1_t());
            subs.push((
                ST::tag,
                SN::E(self.unchecked_advance())
            ))
        }

        subs.push((ST::PathExprSeg, SN::T(self.parse_path_expr_seg()?)));

        while self.peek1_t().check_name("colo2") {
            self.expect_eat_tok1_t(ST::colon2, four)?;
            subs.push((
                ST::PathExprSeg,
                SN::T(self.parse_path_expr_seg()?),
            ));
        }

        Ok(TT::new(subs))
    }

    pub(crate) fn parse_path_expr_seg(&mut self) -> ParseResult2 {
        let four = ST::PathExprSeg;
        let mut subs = vec![];

        let tok = self.expect_eat_id_t(four)?;

        subs.push((ST::id, SN::E(tok)));

        Ok(TT::new(subs))
    }

    pub(crate) fn parse_grouped_expr(&mut self) -> ParseResult2 {
        let four = ST::LitExpr;
        let mut subs = vec![];

        subs.push((
            ST::lparen,
            SN::E(self.expect_eat_tok1_t(ST::lparen, four)?)
        ));

        subs.push((ST::Expr, SN::T(self.parse_expr()?)));

        subs.push((
            ST::rparen,
            SN::E(self.expect_eat_tok1_t(ST::rparen, four)?)
        ));

        Ok(TT::new(subs))
    }

    pub(crate) fn parse_return_expr(&mut self) -> ParseResult2 {
        let four = ST::ReturnExpr;
        let mut subs = vec![
            (ST::ret, SN::E(self.expect_eat_tok1_t(ST::ret, four)?))
        ];

        if !self.peek1_t().check_names_in(&["semi", "rbrace"]) {
            subs.push((ST::Expr, SN::T(self.parse_expr()?)));
        }

        Ok(TT::new(subs))
    }


    pub(crate) fn parse_cmd_expr(&mut self) -> ParseResult2 {
        let four = ST::CmdExpr;
        let mut subs = vec![];

        let cmd_tok = self.expect_eat_tok1_t(ST::cmd, four)?;

        subs.push((ST::cmd, SN::E(cmd_tok)));

        Ok(TT::new(subs))
    }

    pub(crate) fn parse_side_effect_expr(&mut self) -> ParseResult2 {
        let four = ST::SideEffectExpr;

        let tok1 = self.peek1_t();
        let mut subs = vec![];

        if tok1.check_name("id") {
            subs.push((ST::id, SN::E(self.unchecked_advance())));

            if self.peek1_t().check_name("inc") {
                subs.push((ST::inc, SN::E(self.unchecked_advance())));
            } else if self.peek1_t().check_name("dec") {
                subs.push((ST::dec, SN::E(self.unchecked_advance())));
            } else {
                return Err(R::Unrecognized {
                    four,
                    found: self.unchecked_advance(),
                });
            }
        } else if tok1.check_name("inc") {
            subs.push((ST::inc, SN::E(self.unchecked_advance())));
            subs.push((ST::id, SN::E(self.expect_eat_id_t(four)?)));
        } else if tok1.check_name("dec") {
            subs.push((ST::dec, SN::E(self.unchecked_advance())));
            subs.push((ST::id, SN::E(self.expect_eat_id_t(four)?)));
        }

        Ok(TT::new(subs))
    }


    fn try_parse_expr_span(&mut self) -> Result<Option<(ST, SN)>, ParseErrorReason> {
        let tok1 = self.peek1_t().clone();

        Ok(Some(
            // LitExpr
            if let Some((ty, tt)) = self.try_parse_lit_expr() {
                (ty, tt)
            }
            // PathExpr | SideEffectExpr
            else if tok1.check_names_in(&["id", "tag"]) {
                let tok2 = self.peek2_t();
                let ty;
                let tt;

                // 一元操作符不应该操作在有路径前缀的符号上
                if tok2.check_values_in(&["++", "--"]) {
                    ty = ST::SideEffectExpr;
                    tt = SN::T(self.parse_side_effect_expr()?);
                }
                else {
                    ty = ST::PathExpr;
                    tt = SN::T(self.parse_path_expr()?);
                }

                (ty, tt)
            } else if tok1.check_name("continue") {
                // ty = ST::ContinueExpr;
                // tt = SN::T(self.parse_return_expr()?);
                todo!()
            } else if tok1.check_name("break") {
                // ty = ST::BreakExpr;
                // tt = SN::T(self.parse_return_expr()?);
                todo!()
            } else if tok1.check_name("ret") {
                (
                    ST::ReturnExpr,
                    SN::T(self.parse_return_expr()?)
                )
            } else if tok1.check_name("cmd") {
                (
                    ST::CmdExpr,
                    SN::T(self.parse_cmd_expr()?)
                )
            } else {
                return Ok(None)
            }
        ))
    }


    fn try_parse_expr_block(&mut self) -> Result<Option<(ST, SN)>, ParseErrorReason> {
        let tok1 = self.peek1_t();

        Ok(Some(if tok1.check_name("if") {
            (
                ST::IfExpr,
                SN::T(self.parse_if_expr()?)
            )
        } else if tok1.check_name("loop") {
            (
                ST::InfiLoopExpr,
                SN::T(self.parse_infi_loop_expr()?)
            )
        } else if tok1.check_name("while") {
            // item = (ST::InfiLoopExpr, SN::T(self.parse_infi_loop_expr()?))
            todo!()
        } else if tok1.check_name("for") {
            todo!()
        } else if tok1.check_name("lparen") {
            (
                ST::GroupedExpr,
                SN::T(self.parse_grouped_expr()?)
            )
        } else if tok1.check_name("lbrace") {
            (
                ST::BlockExpr,
                SN::T(self.parse_block_expr()?)
            )
        }
        else {
            return Ok(None);
        }))

    }


    fn try_parse_lit_expr(&mut self) -> Option<(ST, SN)> {
        let tok1 = self.peek1_t();

        // LitExpr
        Some(
        if tok1.check_name("lit_char") {  // u32 in memorry
            ST::lit_char
        } else if tok1.check_name("lit_str") {  // char*
            ST::lit_str
        } else if tok1.check_name("lit_rawstr") {  // encoded char*
            ST::lit_rawstr
        } else if tok1.check_name("lit_int") {  // i32
            ST::lit_int
        } else if tok1.check_name("lit_float") {  // f64
            ST::lit_float
        } else if tok1.check_name("lit_bool") {  // u8
            ST::lit_bool
        }
        else {
            return None;
        })
        .map(|ty| (
            ST::LitExpr,
            SN::T(TT::new(vec![(
                ty,
                SN::E(self.unchecked_advance())
            )]))
        ))
    }


    fn try_parse_bop(&mut self) -> Option<BopWrapper<(ST, Token)>> {
        let tok1 = self.peek1_t();

        // Op
        // Precedence 110
        Some(
        if tok1.check_name("as") {
            BopWrapper::new(
                (ST::r#as, self.unchecked_advance()),
                100,
            )
        }
        // Precedence 100
        else if tok1.check_name("mul") {
            BopWrapper::new(
                (ST::mul, self.unchecked_advance()),
                100,
            )
        } else if tok1.check_name("div") {
            BopWrapper::new(
                (ST::div, self.unchecked_advance()),
                100,
            )
        } else if tok1.check_name("percent") {
            BopWrapper::new(
                (ST::percent, self.unchecked_advance()),
                100,
            )
        }
        // Precedence 90
        else if tok1.check_name("add") {
            BopWrapper::new(
                (ST::add, self.unchecked_advance()),
                90,
            )
        } else if tok1.check_name("sub") {
            BopWrapper::new(
                (ST::sub, self.unchecked_advance()),
                90,
            )
        }
        // Precedence 80
        else if tok1.check_name("lshf") {
            BopWrapper::new(
                (ST::lshf, self.unchecked_advance()),
                80,
            )
        } else if tok1.check_name("rshf") {
            BopWrapper::new(
                (ST::rshf, self.unchecked_advance()),
                80,
            )
        }
        // Percedence 70
        else if tok1.check_name("band") {
            BopWrapper::new(
                (ST::band, self.unchecked_advance()),
                70,
            )
        }
        // Percedence 60
        else if tok1.check_name("bxor") {
            BopWrapper::new(
                (ST::bxor, self.unchecked_advance()),
                60,
            )
        }
        // Percedence 50
        else if tok1.check_name("bor") {
            BopWrapper::new(
                (ST::bor, self.unchecked_advance()),
                50,
            )
        }
        // Percedence 40
        else if tok1.check_name("eq") {
            BopWrapper::new(
                (ST::eq, self.unchecked_advance()),
                40,
            )
        } else if tok1.check_name("neq") {
            BopWrapper::new(
                (ST::neq, self.unchecked_advance()),
                40,
            )
        } else if tok1.check_name("gt") {
            BopWrapper::new(
                (ST::gt, self.unchecked_advance()),
                40,
            )
        } else if tok1.check_name("ge") {
            BopWrapper::new(
                (ST::ge, self.unchecked_advance()),
                40,
            )
        } else if tok1.check_name("lt") {
            BopWrapper::new(
                (ST::lt, self.unchecked_advance()),
                40,
            )
        } else if tok1.check_name("le") {
            BopWrapper::new(
                (ST::le, self.unchecked_advance()),
                40,
            )
        }
        // Percedence 30
        else if tok1.check_name("and") {
            BopWrapper::new(
                (ST::and, self.unchecked_advance()),
                30,
            )
        }
        // Percedence 20
        else if tok1.check_name("or") {
            BopWrapper::new(
                (ST::or, self.unchecked_advance()),
                20,
            )
        }
        // Percedence 10 assign
        else if tok1.check_name("assign") {
            BopWrapper::new(
                (ST::assign, self.unchecked_advance()),
                10,
            )
        } else if tok1.check_name("add_assign") {
            BopWrapper::new(
                (ST::add_assign, self.unchecked_advance()),
                10,
            )
        } else if tok1.check_name("sub_assign") {
            BopWrapper::new(
                (ST::sub_assign, self.unchecked_advance()),
                10,
            )
        } else if tok1.check_name("mul_assign") {
            BopWrapper::new(
                (ST::mul_assign, self.unchecked_advance()),
                10,
            )
        } else if tok1.check_name("div_assign") {
            BopWrapper::new(
                (ST::div_assign, self.unchecked_advance()),
                10,
            )
        } else if tok1.check_name("percent_assign") {
            BopWrapper::new(
                (ST::percent_assign, self.unchecked_advance()),
                10,
            )
        }
        // Percedence 0 assign
        else if tok1.check_name("band_assign") {
            BopWrapper::new(
                (ST::band_assign, self.unchecked_advance()),
                0,
            )
        } else if tok1.check_name("bor_assign") {
            BopWrapper::new(
                (ST::bor_assign, self.unchecked_advance()),
                0,
            )
        } else if tok1.check_name("bxor_assign") {
            BopWrapper::new(
                (ST::bxor_assign, self.unchecked_advance()),
                0,
            )
        } else if tok1.check_name("lshf_assign") {
            BopWrapper::new(
                (ST::lshf_assign, self.unchecked_advance()),
                0,
            )
        } else if tok1.check_name("rshf_assign") {
            BopWrapper::new(
                (ST::rshf_assign, self.unchecked_advance()),
                0,
            )
        }
        else {
            return None;
        })
    }
}

fn map_infix_expr_to_tt(
    ie: InfixExpr<BopWrapper<(ST, Token)>, (ST, SN)>,
) -> TT {
    let mut subs = vec![];

    match ie {
        InfixExpr::E(e) => subs.push(e),
        InfixExpr::T { bop, pri1, pri2 } => {
            subs.push((ST::Expr, SN::T(map_infix_expr_to_tt(*pri1))));
            let (bopty, boptt) = bop.unwrap();
            subs.push((bopty, SN::E(boptt)));
            subs.push((ST::Expr, SN::T(map_infix_expr_to_tt(*pri2))));
        }
    }

    TT::new(subs)
}
