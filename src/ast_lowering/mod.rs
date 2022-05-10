mod pass1;
mod pass2;

use std::cmp::{max, min};
use std::rc::Rc;

use indexmap::{indexmap, IndexMap};
use m6coll::Entry;
use m6lexerkit::{str2sym, sym2str, SrcFileInfo, SrcLoc, Symbol, Token};
use pass1::SemanticAnalyzerPass1;

use self::pass2::SemanticAnalyzerPass2;
use crate::opt_osstr_to_str;
use crate::parser::{SyntaxNode, SyntaxType as ST, TokenTree};


////////////////////////////////////////////////////////////////////////////////
//// MIR Data Structure

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum APriType {
    Float(u8), // float64
    Int(i8),   // i32
    Str,       // C string
    // Char,  // u32
    OpaqueStruct(Symbol), // opaque struct pointer type
}

pub(crate) const fn aty_str() -> AType {
    AType::Pri(APriType::Str)
}
pub(crate) const fn aty_int(meta: i8) -> AType {
    AType::Pri(APriType::Int(meta))
}
pub(crate) const fn aty_i32() -> AType {
    AType::Pri(APriType::Int(-4))
}
pub(crate) const fn aty_bool() -> AType {
    AType::Pri(APriType::Int(1))
}
#[allow(unused)]
pub(crate) const fn aty_u8() -> AType {
    AType::Pri(APriType::Int(-4))
}
pub(crate) const fn aty_f64() -> AType {
    AType::Pri(APriType::Float(8))
}
pub(crate) fn aty_opaque_struct(s: &str) -> AType {
    AType::Pri(APriType::OpaqueStruct(str2sym(s)))
}


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(unused)]
pub enum AType {
    Pri(APriType),
    Arr(Vec<APriType>), // Normal Array (usize index)
    AA(Vec<APriType>),  // Associative Array (str index)
    Void,
    PH, // Phantom or Place Holder anyway, used for multiple diagnosis
}

impl AType {
    pub(crate) fn lift_tys(op: ST, ty1: Self, ty2: Self) -> Result<Self, ()> {
        if ty1 == Self::PH || ty2 == Self::PH {
            return Ok(Self::PH);
        }

        match op {
            // It contains risk of overflow
            ST::add | ST::sub | ST::lt | ST::le | ST::gt | ST::ge | ST::eq | ST::neq => {
                match (ty1, ty2) {
                    (Self::Pri(prity1), Self::Pri(prity2)) => {
                        Ok(match (prity1, prity2) {
                            (
                                APriType::Float(_fmeta),
                                APriType::Int(_imeta),
                            ) => aty_f64(),
                            (
                                APriType::Int(_imeta),
                                APriType::Float(_fmeta),
                            ) => aty_f64(),
                            (APriType::Int(imeta1), APriType::Int(imeta2)) => {
                                aty_int({
                                    if imeta1 < 0 && imeta2 < 0 {
                                        min(imeta1, imeta2)
                                    } else if imeta1 > 0 && imeta2 > 0 {
                                        max(imeta1, imeta2)
                                    } else if imeta1 > 0 {
                                        // negative indicates signed
                                        imeta2
                                    } else {
                                        imeta1
                                    }
                                })
                            }
                            (APriType::Str, APriType::Str) => aty_str(),
                            _ => return Err(()),
                        })
                    }
                    _ => Err(()),
                }
            },

            _ => unreachable!("op: {:#?}", op),
        }
    }
}

pub(crate) struct AFn {
    idt: Token, // Identifier Token
    name: Symbol,
    params: Vec<AParamPat>,
    ret: AType,
}

impl std::fmt::Debug for AFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AFn")
            .field("name", &sym2str(self.name))
            .field("params", &self.params)
            .field("ret", &self.ret)
            .finish()
    }
}

impl AFn {}

#[derive(Debug)]
pub(crate) struct AMod {
    name: Symbol,
    afns: IndexMap<Symbol, AFn>,
    scopes: Vec<AScope>, // Start from Root Scope
}

impl AMod {
    fn init(name: Symbol) -> Self {
        Self {
            name,
            afns: indexmap! {},
            scopes: vec![AScope::default()], // push Root Scope
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AVar {
    ty: AType,
    val: AVal, // MIR usize
}

impl AVar {
    pub(crate) fn void() -> Self {
        Self {
            ty: AType::Void,
            val: AVal::PH,
        }
    }

    pub(crate) fn undefined() -> Self {
        Self {
            ty: AType::PH,
            val: AVal::PH,
        }
    }
}

impl Default for AVar {
    fn default() -> Self {
        Self::void()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ConstVal {
    Int(i32),
    Float(f64),
    Str(Symbol),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub(crate) enum AVal {
    IfBlock {
        if_exprs: Vec<(Symbol, usize)>, // (cond, then-block)
        else_blk: Option<usize>,        // else block (scope idx)
    },
    BlockExpr(usize), // Scope idx
    FnParam(usize),
    FnCall {
        call_fn: Symbol,
        args: Vec<Symbol>,
    },
    BOpExpr {
        op: Option<ST>,
        operands: Vec<Symbol>,
    },
    ConstAlias(ConstVal),
    Break,
    Continue,
    Return(Option<Symbol>),
    PH,
}

impl AVal {
    pub(crate) fn as_block_expr_idx(&self) -> usize {
        match self {
            Self::BlockExpr(idx) => *idx,
            _ => unreachable!("{:#?}", self),
        }
    }
}

#[derive(Debug)]
pub(crate) struct MIR {
    // SSA
    name: Symbol,
    ty: AType,
    val: AVal,
}

impl MIR {
    fn undefined(name: Symbol) -> Self {
        Self {
            name,
            ty: AType::PH,
            val: AVal::PH,
        }
    }

    fn side_effect(val: AVal) -> Self {
        Self {
            name: str2sym(""),
            ty: AType::Void,
            val,
        }
    }

    fn bind(name: Symbol, var: AVar) -> Self {
        Self {
            name,
            ty: var.ty,
            val: var.val,
        }
    }

    fn var(&self) -> AVar {
        AVar {
            ty: self.ty.clone(),
            val: self.val.clone(),
        }
    }
}

#[derive(Default)]
pub(crate) struct AScope {
    paren: Option<usize>, // Scope id, 0 means root, -1 means None
    explicit_bindings: Vec<Entry<Symbol, usize>>,
    implicit_bindings: IndexMap<Symbol, usize>,
    mirs: Vec<MIR>,
}

impl AScope {
    pub(crate) fn tmp_name(&self) -> Symbol {
        str2sym(&format!("!__tmp_{}", self.implicit_bindings.len()))
    }

    pub(crate) fn ret(&self) -> AVar {
        if self.mirs.is_empty() {
            AVar::void()
        } else {
            let last_mir = self.mirs.last().unwrap();
            last_mir.var()
        }
    }

    pub(crate) fn in_scope_find_sym(&self, q: &Symbol) -> Option<AVar> {
        self.explicit_bindings
            .iter()
            .rev()
            .find(|Entry(sym, _mir_idx)| sym == q)
            .and_then(|Entry(_sym, mir_idx)| Some(self.mirs[*mir_idx].var()))
    }

    // pub(crate) fn in_scope_find_sym(&self, q: &Symbol) -> Option<AVar> {
    //     self.explicit_bindings
    //     .iter()
    //     .rev()
    //     .find(|Entry(sym, mir_idx)| sym == q)
    //     .and_then(|Entry(sym, mir_idx)| Some(self.mirs[*mir_idx].var()))
    // }
}

impl std::fmt::Debug for AScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct DVec(Vec<Entry<Symbol, usize>>);
        impl std::fmt::Debug for DVec {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                for Entry(sym, _idx) in self.0.iter() {
                    writeln!(f, "{}", sym2str(*sym))?;
                }

                Ok(())
            }
        }

        f.debug_struct("AScope")
            .field("paren", &self.paren)
            .field("explicit_bindings", &DVec(self.explicit_bindings.clone()))
            .field("implicit_bindings", &self.implicit_bindings)
            .field("mirs", &self.mirs)
            .finish()
    }
}



#[derive(Debug)]
pub(crate) struct AParamPat {
    formal: Symbol,
    ty: AType,
}

impl AParamPat {
    // fn fake() -> Self {
    //     Self {
    //         formal: str2sym(""),
    //         ty: AType::PH,
    //     }
    // }
}


////////////////////////////////////////////////////////////////////////////////
//// Diagnosis

pub(crate) struct SemanticError {
    diagnosis: Vec<DiagnosisItem>,
}
impl std::fmt::Debug for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in self.diagnosis.iter() {
            writeln!(f, "{}, {:#?}", item.loc, item.dtype)?;
            writeln!(f, "{}", "-".to_string().repeat(80))?;
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

pub(crate) struct DiagnosisItem {
    dtype: DiagnosisType,
    loc: SrcLoc,
}
pub(crate) struct DiagnosisItem2 {
    dtype: DiagnosisType,
    tok: Token,
}
impl DiagnosisItem2 {
    fn into_d1(self, src: &SrcFileInfo) -> DiagnosisItem {
        DiagnosisItem {
            dtype: self.dtype,
            loc: src.boffset2srcloc(self.tok.span.from),
        }
    }
}

pub enum DiagnosisType {
    DupItemDef { name: Symbol, prev: Token },
    LackFormalParam {},
    IncompatiableOpType { op1: AType, op2: AType },
    IncompatiableIfExprs { if1: AType, oths: Vec<AType> },
    UnknownSymbolBinding(Symbol),
    UnsupportedStringifyType(AType),
}

impl std::fmt::Debug for DiagnosisType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DupItemDef { name, prev } => f
                .debug_struct("DupItemDef")
                .field("name", &sym2str(*name))
                .field("prev", prev)
                .finish(),
            Self::LackFormalParam {} => {
                f.debug_struct("LackFormalParam").finish()
            }
            Self::IncompatiableOpType { op1, op2 } => f
                .debug_struct("IncompatiableOpType")
                .field("op1", op1)
                .field("op2", op2)
                .finish(),
            Self::IncompatiableIfExprs { if1, oths } => f
                .debug_struct("IncompatiableIfExprs")
                .field("if1", if1)
                .field("oths", oths)
                .finish(),
            Self::UnknownSymbolBinding(arg0) => f
                .debug_tuple("UnknownSymbolBinding")
                .field(&sym2str(*arg0))
                .finish(),
            Self::UnsupportedStringifyType(arg0) => f
                .debug_tuple("UnsupportedStringifyType")
                .field(arg0)
                .finish(),
        }
    }
}

pub(crate) type AnalyzeResult = Result<AMod, SemanticError>;
pub(crate) type AnalyzeResult2 = Result<AMod, Vec<DiagnosisItem2>>;


////////////////////////////////////////////////////////////////////////////////
//// SyntaxNode Implements

impl SyntaxNode {
    pub(crate) fn as_tt(&self) -> &TokenTree {
        match self {
            SyntaxNode::T(ref tt) => tt,
            SyntaxNode::E(_) => unreachable!("{:?}", self),
        }
    }

    pub(crate) fn as_tok(&self) -> &Token {
        match self {
            SyntaxNode::T(_) => unreachable!("{:?}", self),
            SyntaxNode::E(ref tok) => tok,
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Semantic Analyze

fn _semantic_analyze(
    tt: TokenTree,
    src: &SrcFileInfo,
) -> Result<AMod, Vec<DiagnosisItem2>> {
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
        Err(ditems) => {
            let diagnosis =
                ditems.into_iter().map(|ditem| ditem.into_d1(src)).collect();

            Err(SemanticError { diagnosis })
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
        let path = PathBuf::from("./examples/exp0.bath");
        let src = SrcFileInfo::new(&path).unwrap();

        // println!("{:#?}", sp_m(srcfile.get_srcstr(), SrcLoc { ln: 0, col: 0 }));

        let tokens = tokenize(&src)?;
        let tt = parse(tokens, &src)?;
        let amod = semantic_analyze(tt, &src)?;

        println!("{:#?}", amod);

        Ok(())
    }
}
