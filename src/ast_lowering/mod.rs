pub mod data;
mod pass1;
mod pass2;


use std::fmt::Debug;
use std::rc::Rc;

use itertools::Itertools;
use m6lexerkit::{str2sym, sym2str, Span, SrcFileInfo, Symbol, Token};
use pass1::SemanticAnalyzerPass1;

pub use self::data::*;
use self::pass2::SemanticAnalyzerPass2;
use crate::name_mangling::mangling;
use crate::parser::{SyntaxNode as SN, SyntaxType as ST, TokenTree};
use crate::{opt_osstr_to_str, ref_source};




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
        prev: Span,
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
    #[allow(unused)]
    UnmatchedType(AType, AType),
    UnkonwnType,
    NoMatchedFunc(Symbol, Vec<AType>), // basename, tys
}
use SemanticErrorReason as R;

pub(crate) type CauseLists = Vec<(R, Span)>;
pub(crate) type AnalyzeResult = Result<AMod, SemanticError>;
pub(crate) type AnalyzeResult2 = Result<AMod, CauseLists>;


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
                "-".to_string().repeat(34),
            )?;
            writeln!(f)?;

            // writeln!(f, "{}, {:#?}", item.loc, item.dtype)?;
            match cause {
                R::DupItemDef { name, prev } => {
                    writeln!(
                        f,
                        "Duplicate item `{}` definition:\n",
                        sym2str(*name)
                    )?;
                    /* frontwards reference */
                    ref_source!(prev, "=", f, self.src);
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
                R::UnmatchedType(var, val) => writeln!(
                    f,
                    "Unmatched Type variable: {:?}, value: {:?}:\n",
                    var, val
                ),
                R::UnkonwnType => writeln!(f, "Unknown Type:\n"),
                R::NoMatchedFunc(basename, tys) => {
                    let fullname_sym = mangling(*basename, tys);

                    writeln!(f, "No def for {}", sym2str(fullname_sym))
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


impl SN {
    pub(crate) fn as_tt(&self) -> &TokenTree {
        match self {
            Self::T(ref tt) => tt,
            SN::E(_) => unreachable!("{:?}", self),
        }
    }

    pub(crate) fn as_tok(&self) -> &Token {
        match self {
            Self::T(_) => unreachable!("{:?}", self),
            Self::E(ref tok) => tok,
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Function




fn _semantic_analyze(tt: TokenTree, src: &SrcFileInfo) -> AnalyzeResult2 {
    let amod =
        AMod::init(str2sym(opt_osstr_to_str!(&src.get_path().file_stem())));
    let tt = Rc::new(tt);

    let pass1 = SemanticAnalyzerPass1::new(amod, tt.clone());
    let amod = pass1.analyze()?;

    let pass2 = SemanticAnalyzerPass2::new(amod, tt.clone());
    let amod = pass2.analyze()?;

    Ok(amod)
}

pub(crate) fn semantic_analyze(
    tt: TokenTree,
    src: &SrcFileInfo,
) -> AnalyzeResult {
    match _semantic_analyze(tt, src) {
        Ok(amod) => Ok(amod),
        Err(cause_lists) => Err(SemanticError {
            cause_lists,
            src: src.clone(),
        }),
    }
}

////////////////////////////////////////
//// Function shared betweem passes

pub(crate) fn get_fullname_by_fn_header(
    base: Symbol,
    params: &[AParamPat],
) -> Symbol {
    // 作为特殊的入口，main最多只有一个实现
    if sym2str(base) == "main".to_owned() {
        base
    } else {
        let tys = params
            .into_iter()
            .map(|param| param.ty.clone())
            .collect_vec();

        mangling(base, &tys)
    }
}

pub(crate) fn analyze_fn_params(
    cause_lists: &mut CauseLists,
    tt: &TokenTree,
) -> Vec<AParamPat> {
    let mut sns = tt.subs.iter().peekable();
    let mut params = vec![];

    while !sns.is_empty() && sns.peek().unwrap().0 == ST::FnParam {
        let (param_ty, param_sn) = sns.next().unwrap();

        if *param_ty == ST::id {
            write_diagnosis(
                cause_lists,
                R::LackFormalParam,
                param_sn.as_tok().span(),
            );
        }

        params.push(analyze_fn_param(cause_lists, param_sn.as_tt()));
    }

    params
}

pub(crate) fn analyze_fn_param(
    cause_lists: &mut CauseLists,
    tt: &TokenTree,
) -> AParamPat {
    // PatNoTop
    let formal = analyze_pat_no_top(tt[0].1.as_tt());
    let ty = analyze_ty(cause_lists, &tt[1].1.as_tt());

    AParamPat { formal, ty }
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
        return Ok(AType::Pri(APriType::Str));
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



#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use m6lexerkit::SrcFileInfo;

    use crate::{
        ast_lowering::semantic_analyze, lexer::tokenize, parser::parse,
    };

    #[test]
    fn test_analyze() -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from("./examples/exp1.bath");
        let src = SrcFileInfo::new(&path).unwrap();

        let tokens = tokenize(&src)?;
        let tt = parse(tokens, &src)?;
        let amod = semantic_analyze(tt, &src)?;

        println!("{:#?}", amod);

        Ok(())
    }
}
