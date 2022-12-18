#![feature(box_syntax)]
#![feature(exact_size_is_empty)]
#![feature(slice_index_methods)]

//! 特别地，允许 `mutable_borrow_reservation_conflict`
//! 迁就这个问题会很不方便地影响熟悉的控制流，而这个原始问题最新的[消息]
//! (https://github.com/rust-lang/rust/issues/56254) 还是2019年
//! 而且还停留在调查阶段，现在（2022年）几乎没有任何理由认为这个进程会被突然推进，
//! 至少edition 2021绝对不会出问题
//!
//! 现在已经通过了
// #![allow(mutable_borrow_reservation_conflict)]

mod lexer;
mod parser;
pub mod driver;
mod ast_lowering;
mod codegen;
pub mod shell;
pub mod env;
pub(crate) mod aux;
pub(crate) mod name_mangling;
pub mod spec;

