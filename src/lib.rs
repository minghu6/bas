#![feature(box_syntax)]
#![feature(exact_size_is_empty)]

//! 特别地，允许 `mutable_borrow_reservation_conflict`
//! 迁就这个问题会很不方便地影响熟悉的控制流，而这个原始问题最新的[消息]
//! (https://github.com/rust-lang/rust/issues/56254) 还是2019年
//! 而且还停留在调查阶段，现在（2022年）几乎没有任何理由认为这个进程会被突然推进，
//! 至少edition 2021绝对不会出问题
#![allow(mutable_borrow_reservation_conflict)]

mod lexer;
mod parser;
pub mod driver;
mod ast_lowering;
pub mod shell;
pub(crate) mod aux;
