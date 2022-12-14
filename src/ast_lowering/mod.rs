pub mod data;
pub mod pass1;
pub mod pass2;


use std::fmt::Debug;

use itertools::Itertools;
use m6lexerkit::{sym2str, Span, SrcFileInfo, Symbol, Token};

use crate::{
    name_mangling::mangling,
    parser::{SyntaxNode as SN, SyntaxType as ST, TokenTree},
    ref_source,
};
pub use pass1::SemanticAnalyzerPass1;
pub use pass2::SemanticAnalyzerPass2;
pub use self::data::*;


pub struct TokenTree2 {
    items: Vec<AnItem>
}


pub enum AnItem {
    Fn {
        name: Symbol,
        body: TokenTree
    }
}


#[derive(Debug, Clone)]
pub(crate) struct MIR {
    // SSA
    pub(crate) name: Symbol,
    pub(crate) tagid: Option<usize>,
    pub(crate) mirty: MIRTy,
    pub(crate) ty: AType,
    pub(crate) val: AVal,
}


#[derive(Debug, Clone)]
pub(crate) enum MIRTy {
    ValBind,
    VarAssign,
}


pub enum SemanticErrorReason {
    DupItemDef {
        name: Symbol,
    },
    LackFormalParam,
    IncompatOpType {
        op1: AType,
        op2: AType,
    },
    IncompatIfExprs {
        if1: AType,
        oths: Vec<AType>,
    },
    UnknownSymBinding(Symbol),
    CantCastType(AType, AType),
    /// expect, found, for "..."
    UnmatchedType(AType, AType, String),
    UnkonwnType,
    UnkonwTag,
    NoMatchedFunc(Symbol, Vec<AType>), // basename, tys
    DuplicateAttr(Symbol, A3ttrVal),
    UnknownAttr(Symbol),
}
use SemanticErrorReason as R;

pub(crate) type CauseLists = Vec<(R, Span)>;


pub struct SemanticError {
    src: SrcFileInfo,
    cause_lists: Vec<(SemanticErrorReason, Span)>,
}


impl MIR {
    fn bind_value(name: Symbol, var: AVar) -> Self {
        Self {
            name,
            tagid: None,
            mirty: MIRTy::ValBind,
            ty: var.ty,
            val: var.val,
        }
    }

    fn assign_var(name: Symbol, tagid: usize, var: AVar) -> Self {
        Self {
            name,
            tagid: Some(tagid),
            mirty: MIRTy::VarAssign,
            ty: var.ty,
            val: var.val,
        }
    }
}


////////////////////////////////////////
//// Diagnosis Implement

impl std::fmt::Debug for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(f)?;

        for (i, (cause, span)) in self.cause_lists.iter().enumerate() {
            writeln!(
                f,
                "{} cause-{:003} {}",
                "-".to_string().repeat(34),
                i + 1,
                "-".to_string().repeat(36),
            )?;
            writeln!(f)?;

            // writeln!(f, "{}, {:#?}", item.loc, item.dtype)?;
            match cause {
                R::DupItemDef { name } => {
                    writeln!(
                        f,
                        "Duplicate item `{}` definition:\n",
                        sym2str(*name)
                    )?;
                    /* frontwards reference */
                    // ref_source!(prev, "=", f, self.src);
                    Ok(())
                }
                R::LackFormalParam => {
                    writeln!(f, "Lack formal param:\n")
                }
                R::IncompatOpType { op1, op2 } => {
                    writeln!(f, "No compatiable operator between {op1:?} and {op2:?}:\n")
                }
                R::IncompatIfExprs { if1, oths } => {
                    writeln!(f, "If block type {if1:?} diffs in {oths:#?}:\n",)
                }
                R::UnknownSymBinding(arg0) => {
                    writeln!(f, "Unkonwn symbol {}:\n", sym2str(*arg0))
                }
                // R::UnsupportedStringifyType(arg0) => {
                //     writeln!(f, "Can't stringify {arg0:?}:\n")
                // }
                R::CantCastType(from, to) => {
                    writeln!(f, "Can't cast {:?} into {:?}:\n", from, to)
                }
                R::UnmatchedType(expect, found, four) => writeln!(
                    f,
                    "Expect Type {:?}, however found {:?} for {}:\n",
                    expect, found, four
                ),
                R::UnkonwnType => writeln!(f, "Unknown Type:\n"),
                R::NoMatchedFunc(basename, tys) => {
                    let fullname_sym = mangling(*basename, tys);

                    writeln!(f, "No def for {}", sym2str(fullname_sym))
                }
                R::DuplicateAttr(attrsym, attrval) => {
                    writeln!(
                        f,
                        "Duplicate Attr annotation {}, prev: {attrval:?}",
                        sym2str(*attrsym)
                    )
                }
                R::UnknownAttr(attrsym) => {
                    writeln!(
                        f,
                        "Unknown Attr annotation {}",
                        sym2str(*attrsym)
                    )
                },
                R::UnkonwTag => {
                    writeln!(
                        f,
                        "Unknown Tag"
                    )
                }
            }?;
            writeln!(f)?;
            ref_source!(span, "^", f, self.src);
            writeln!(f)?;
            writeln!(f)?;
        }

        Ok(())
    }
}


impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SemanticError {}


impl TokenTree {
    pub(crate) fn move_elem(&mut self, i: usize) -> (ST, SN) {
        std::mem::replace(&mut self.subs[i], (ST::semi, SN::E(Token::eof())))
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Function


////////////////////////////////////////
//// Function shared betweem passes

pub(crate) fn calc_fullname(
    base: Symbol,
    params: &[AParamPat],
) -> Symbol {
    let tys = params
        .into_iter()
        .map(|param| param.ty.clone())
        .collect_vec();

    mangling(base, &tys)
}

pub(crate) fn analyze_pat_no_top(tt: &TokenTree) -> Symbol {
    let id = tt[0].1.as_tok();

    id.value
}

pub(crate) fn analyze_ty(
    cause_lists: &mut CauseLists,
    tt: &TokenTree,
) -> AType {
    match analyze_ty_(tt) {
        Ok(aty) => aty,
        Err(span) => {
            write_diagnosis(cause_lists, R::UnkonwnType, span);
            AType::PH
        }
    }
}


pub(crate) fn analyze_ty_(tt: &TokenTree) -> Result<AType, Span> {
    let tok_id = tt[0].1.as_tok();

    // analyze alias -- skip (inner multiple scan)
    if tok_id.check_value("int") {
        return Ok(aty_i32());
    }
    if tok_id.check_value("float") {
        return Ok(aty_f64());
    }
    if tok_id.check_value("str") {
        return Ok(AType::Pri(APriType::Ptr));
    }
    if tok_id.check_value("ptr") {
        return Ok(AType::Pri(APriType::Ptr));
    }
    if tok_id.check_value("[") {
        if tt.len() < 2 {
            return Err(tok_id.span);
        }

        let tok2 = &tt[1].1.as_tok();

        return match tok2.value_string().as_str() {
            "int" => Ok(AType::Arr(APriType::Int(-4), 1)),
            "float" => Ok(AType::Arr(APriType::Float(8), 1)),
            _ => {
                if tt.len() < 3 {
                    return Err(Span {
                        from: tok_id.span.from,
                        end: tok2.span.end,
                    });
                }
                let tok3 = &tt[2].1.as_tok();
                return Err(Span {
                    from: tok_id.span.from,
                    end: tok3.span.end,
                });
            }
        };
    }

    Err(tok_id.span)
}


pub(crate) fn analyze_attrs(
    cause_lists: &mut CauseLists,
    tt: &TokenTree,
) -> A3ttrs {
    let mut attrs = A3ttrs::new();

    for (st, sn) in tt.iter() {
        debug_assert_eq!(*st, ST::attr);

        let idt = sn.as_tok();

        let attr_name = match idt.value_string().as_str() {
            "no_mangle" => A3ttrName::NoMangle,
            "vararg" => A3ttrName::VarArg,
            _ => {
                write_diagnosis(
                    cause_lists,
                    R::UnknownAttr(idt.value),
                    idt.span,
                );
                return attrs;
            }
        };

        if let Some(oldval) = attrs.0.insert(attr_name, A3ttrVal::Empty) {
            write_diagnosis(
                cause_lists,
                R::DuplicateAttr(idt.name, oldval),
                idt.span,
            )
        }
    }

    attrs
}


pub(crate) fn write_diagnosis(
    cause_lists: &mut Vec<(R, Span)>,
    r: R,
    span: Span,
) {
    cause_lists.push((r, span))
}

#[allow(unused)]
pub(crate) fn toks_to_span(toks: &[Token]) -> Span {
    if toks.len() == 0 {
        Span::default()
    } else if toks.len() == 1 {
        toks[0].span
    } else {
        Span {
            from: toks[0].span.from,
            end: toks.last().unwrap().span.end,
        }
    }
}
