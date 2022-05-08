mod pass1;
mod pass2;

use std::rc::Rc;

use indexmap::{ IndexMap, indexmap };
use m6coll::Entry;
use m6lexerkit::{str2sym, SrcFileInfo, SrcLoc, Symbol, Token};
use pass1::SemanticAnalyzerPass1;

use crate::opt_osstr_to_str;
use crate::parser::{SyntaxNode, TokenTree, SyntaxType};

use self::pass2::SemanticAnalyzerPass2;


////////////////////////////////////////////////////////////////////////////////
//// MIR Data Structure

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum APriType {
    F64, // float64
    Int, // i32
    Str, // C string
    // Char,  // u32
    Bool,  // u8
}

pub(crate) const fn aty_str() -> AType {
    AType::Pri(APriType::Str)
}
pub(crate) const fn aty_int() -> AType {
    AType::Pri(APriType::Int)
}
pub(crate) const fn aty_f64() -> AType {
    AType::Pri(APriType::F64)
}


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AType {
    Pri(APriType),
    Arr(Vec<APriType>),  // Normal Array (usize index)
    AA(Vec<APriType>),   // Associative Array (str index)
    Void,
    PH  // Phantom or Place Holder anyway, used for multiple diagnosis
}

impl AType {
    pub(crate) fn lift_tys(ty1: &Self, ty2: &Self) -> Result<Self, ()> {
        match (ty1, ty2) {
            (Self::Pri(prity1), Self::Pri(prity2)) =>
            match (prity1, prity2) {
                (APriType::F64, APriType::F64) => Ok(Self::Pri(APriType::F64)),
                (APriType::F64, APriType::Int) => Ok(Self::Pri(APriType::F64)),
                (APriType::Int, APriType::F64) => Ok(Self::Pri(APriType::F64)),
                (APriType::Int, APriType::Int) => Ok(Self::Pri(APriType::Int)),
                (APriType::Str, APriType::F64) => Err(()),
                (APriType::Str, APriType::Int) => Err(()),
                (APriType::Str, APriType::Str) => Ok(Self::Pri(APriType::Str)),
                (APriType::Bool, APriType::Bool) => Ok(Self::Pri(APriType::Bool)),
                (_, _) => Err(()),

                // (APriType::Bool, APriType::Int) => Ok(Self::Pri(APriType::Bool)),
            },
            (Self::Pri(_), Self::PH) => Ok(Self::PH),
            (Self::Pri(_), _) => Err(()),
            (Self::Arr(_), _) => todo!(),
            (Self::AA(_), _) => todo!(),
            (Self::Void, _) => Err(()),
            (Self::PH, _) => Ok(Self::PH),
        }
    }


}

pub(crate) struct AFn {
    idt: Token,  // Identifier Token
    name: Symbol,
    params: Vec<AParamPat>,
    ret: AType,
}

impl AFn {}

pub(crate) struct AMod {
    name: Symbol,
    afns: IndexMap<Symbol, AFn>,
    scopes: Vec<AScope>,  // Start from Root Scope
}

impl AMod {
    fn init(name: Symbol) -> Self {
        Self {
            name,
            afns: indexmap!{},
            scopes: vec![AScope::default()]  // push Root Scope
        }
    }
}

pub(crate) struct AVar {
    ty: AType,
    val: AVal  // MIR usize
}

impl AVar {
    pub(crate) fn void() -> Self {
        Self {
            ty: AType::Void,
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
    Bool(bool)
}

#[derive(Debug, Clone)]
pub(crate) enum AVal {
    IfBlock {
        if_exprs: Vec<(Symbol, usize)>,  // (cond, then-block)
        else_blk: Option<usize>  // else block (scope idx)
    },
    BlockExpr(usize),  // Scope idx
    FnParam(usize),
    FnCall {
        call_fn: Symbol,
        args: Vec<Symbol>
    },
    BOpExpr {
        op: Option<SyntaxType>,
        operands: Vec<Symbol>
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
            _ => unreachable!("{:#?}", self)
        }
    }
}

pub(crate) struct MIR {  // SSA
    name: Symbol,
    ty: AType,
    val: AVal
}

impl MIR {
    fn undefined(name: Symbol) -> Self {
        Self {
            name,
            ty: AType::PH,
            val: AVal::PH,
        }
    }

    fn side_effect(val :AVal) -> Self {
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
        AVar { ty: self.ty.clone(), val: self.val.clone() }
    }
}

#[derive(Default)]
pub(crate) struct AScope {
    paren: Option<usize>,  // Scope id, 0 means root, -1 means None
    explicit_bindings: Vec<Entry<Symbol, usize>>,
    implicit_bindings: IndexMap<Symbol, usize>,
    mirs: Vec<MIR>,
}

impl AScope {
    pub(crate) fn tmp_name(&self) -> Symbol {
        str2sym(&format!("!__tmp_{}", self.implicit_bindings.len()))
    }

    pub(crate) fn name_var(&mut self, var: AVar) -> Symbol {
        let tmp = self.tmp_name();
        self.mirs.push(MIR::bind(tmp, var));
        self.implicit_bindings.insert(tmp, self.mirs.len() - 1);

        tmp
    }

    pub(crate) fn ret(&self) -> AVar {
        if self.mirs.is_empty() {
            AVar::void()
        }
        else {
            let last_mir = self.mirs.last().unwrap();
            last_mir.var()
        }
    }

    pub(crate) fn in_scope_find_sym(&self, q: &Symbol) -> Option<AVar> {
        self.explicit_bindings
        .iter()
        .rev()
        .find(|Entry(sym, mir_idx)| sym == q)
        .and_then(|Entry(sym, mir_idx)| Some(self.mirs[*mir_idx].var()))
    }
}


pub(crate) struct AParamPat {
    formal: Symbol,
    ty: AType
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

#[derive(Debug)]
pub enum DiagnosisType {
    DupItemDef {
        name: Symbol,
        prev: Token
    },
    LackFormalParam {
    },
    IncompatiableOpType {
        op1: AType,
        op2: AType
    },
    IncompatiableIfExprs {
        if1: AType,
        oths: Vec<AType>
    },
    UnknownSymbolBinding(Symbol),
    UnsupportedStringifyType(AType)
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
            let diagnosis = ditems
            .into_iter()
            .map(|ditem| ditem.into_d1(src))
            .collect();

            Err(SemanticError { diagnosis })
        },
    }
}
