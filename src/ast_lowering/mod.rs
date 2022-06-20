mod pass1;
mod pass2;

use std::cmp::{max, min};
use std::fmt::Debug;
use std::rc::Rc;

use indexmap::{indexmap, IndexMap};
use inkwellkit::get_ctx;
use inkwellkit::types::{FloatType, IntType};
use m6coll::Entry;
use m6lexerkit::{str2sym0, sym2str, SrcFileInfo, SrcLoc, Symbol, Token, Span};
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

impl APriType {
    pub(crate) fn as_float_ty<'ctx>(&self) -> FloatType<'ctx> {
        let ctx = get_ctx();
        match self {
            Self::Float(i8) => {
                match i8 {
                    4 => ctx.f32_type(),
                    8 => ctx.f64_type(),
                    _ => unimplemented!()
                }
            },
            _ => unreachable!(),
        }
    }

    pub(crate) fn as_int_ty<'ctx>(&self) -> IntType<'ctx> {
        let ctx = get_ctx();
        match self {
            Self::Int(i8) => {
                match i8 {
                    4 => ctx.i32_type(),
                    8 => ctx.i64_type(),
                    _ => unimplemented!()
                }
            },
            _ => unreachable!(),
        }
    }
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
    AType::Pri(APriType::OpaqueStruct(str2sym0(s)))
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

    pub(crate) fn try_cast(&self, ty: &Self) -> Result<(), ()> {
        if self == ty {
            return Ok(())
        }

        match (self, ty) {
            (Self::Pri(prity1), Self::Pri(prity2)) => {
                Ok(match (prity1, prity2) {
                    (APriType::Float(_fmeta), APriType::Int(_imeta)) => (),
                    (APriType::Int(_imeta1), APriType::Int(_imeta2)) => (),
                    _ => return Err(())
                })
            }
            _ => Err(())
        }
    }

}

pub(crate) struct AFnDec {
    // idt: Token,  // Identifier Token
    // body_idx: Option<usize>,
    pub(crate) name: Symbol,
    pub(crate) params: Vec<AParamPat>,
    pub(crate) ret: AType,
}

impl std::fmt::Debug for AFnDec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AFn")
            .field("name", &sym2str(self.name))
            .field("params", &self.params)
            .field("ret", &self.ret)
            .finish()
    }
}

impl AFnDec {}

pub(crate) type AFnAlloc = IndexMap<(Symbol, usize), AType>;

pub(crate) struct AMod {
    pub(crate) name: Symbol,
    pub(crate) afns: IndexMap<Symbol, AFnDec>,
    pub(crate) allocs: IndexMap<Symbol, AFnAlloc>,
    pub(crate) scopes: Vec<AScope>, // Start from Root Scope
}

impl Debug for AMod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct AScopeVec<'a>(&'a Vec<AScope>);
        impl<'a> std::fmt::Debug for AScopeVec<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                for (i, ascope) in self.0.iter().enumerate() {
                    writeln!(f, "{} {} {}", "-".repeat(35), i, "-".repeat(35))?;
                    writeln!(f, "{:#?}", ascope)?;
                }

                Ok(())
            }
        }

        f.debug_struct("AMod")
        .field("name", &self.name)
        .field("afns", &self.afns)
        .field("allocs", &self.allocs)
        .field("scopes", &AScopeVec(&self.scopes))
        .finish()
    }
}


impl AMod {
    fn init(name: Symbol) -> Self {
        Self {
            name,
            afns: indexmap! {},
            allocs: indexmap! {},
            scopes: vec![AScope::default()], // push Root Scope
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AVar {
    pub(crate) ty: AType,
    pub(crate) val: AVal, // MIR usize
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
pub(crate) struct ASymDef {
    pub(crate) name: Symbol,
    pub(crate) ty: AType,
}

impl ASymDef {
    pub(crate) fn new(name: Symbol, ty: AType) -> Self {
        Self {
            name,
            ty,
        }
    }

    pub(crate) fn undefined() -> Self {
        Self {
            name: str2sym0(""),
            ty: AType::PH,
        }
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
    DefFn {
        name: Symbol,
        scope_idx: usize
    },
    IfBlock {
        if_exprs: Vec<(Symbol, usize)>, // (cond, then-block)
        else_blk: Option<usize>,        // else block (scope idx)
    },
    InfiLoopExpr(usize),
    BlockExpr(usize), // Scope idx
    FnParam(u32),
    FnCall {
        call_fn: Symbol,
        args: Vec<Symbol>,
    },
    BOpExpr {
        op: ST,
        operands: Vec<Symbol>,
    },
    TypeCast {
        name: Symbol,
        ty: AType
    },
    ConstAlias(ConstVal),
    Var(Symbol, usize),  // symname, tagid : get var value
    Assign(Symbol, usize, Symbol),  //  namesym, tagid, valsym : set var value
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

    pub(crate) fn as_var(&self) -> (Symbol, usize) {
        match self {
            Self::Var(sym, tagid) => (*sym, *tagid),
            _ => unreachable!("{:#?}", self),
        }
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
    VarAssign
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


#[derive(Default)]
pub(crate) struct AScope {
    pub(crate) paren: Option<usize>,
    /// val: tagid, ty
    pub(crate) explicit_bindings: Vec<Entry<Symbol, (usize, AType)>>,
    pub(crate) implicit_bindings: IndexMap<Symbol, usize>,
    pub(crate) mirs: Vec<MIR>,
    pub(crate) ret: Option<AVar>
}

impl AScope {
    pub(crate) fn tmp_name(&self) -> Symbol {
        str2sym0(&format!("!__tmp_{}", self.implicit_bindings.len()))
    }

    pub(crate) fn ret(&self) -> AVar {
        if let Some(ret) = self.ret.clone() {
            ret
        } else {
            AVar::void()
        }
    }

    pub(crate) fn in_scope_find_sym(&self, q: &Symbol) -> Option<(usize, AType)> {
        self.explicit_bindings
            .iter()
            .rev()
            .find(|Entry(sym, _mir_idx)| sym == q)
            .and_then(|Entry(_sym, (tagid, ty))| Some((*tagid, ty.clone())))
    }
}


impl std::fmt::Debug for AScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // struct DVec(Vec<Entry<Symbol, usize>>);
        // impl std::fmt::Debug for DVec {
        //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //         for Entry(sym, _idx) in self.0.iter() {
        //             writeln!(f, "{}", sym2str(*sym))?;
        //         }

        //         Ok(())
        //     }
        // }

        f.debug_struct("AScope")
            .field("paren", &self.paren)
            .field("explicit_bindings", &self.explicit_bindings)
            .field("implicit_bindings", &self.implicit_bindings)
            .field("mirs", &self.mirs)
            .field("ret", &self.ret)
            .finish()
    }
}



#[derive(Debug)]
pub(crate) struct AParamPat {
    pub(crate) formal: Symbol,
    pub(crate) ty: AType,
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
    span: Span,
}
impl DiagnosisItem2 {
    fn into_d1(self, src: &SrcFileInfo) -> DiagnosisItem {
        DiagnosisItem {
            dtype: self.dtype,
            loc: src.boffset2srcloc(self.span.from),
        }
    }
}

pub enum DiagnosisType {
    DupItemDef { name: Symbol, prev: Span },
    LackFormalParam {},
    IncompatiableOpType { op1: AType, op2: AType },
    IncompatiableIfExprs { if1: AType, oths: Vec<AType> },
    UnknownSymbolBinding(Symbol),
    UnsupportedStringifyType(AType),
    CantCastType(AType, AType),
    UnmatchedType(AType, AType)
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
            Self::CantCastType(from, to) =>
                write!(f, "Can't cast {:?} into {:?}", from ,to),
            Self::UnmatchedType(var, val) =>
                write!(f, "Unmatched Type  variable: {:?}, value: {:?}", var, val)
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
        AMod::init(str2sym0(opt_osstr_to_str!(&src.get_path().file_stem())));
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

        let tokens = tokenize(&src)?;
        let tt = parse(tokens, &src)?;
        let amod = semantic_analyze(tt, &src)?;

        println!("{:#?}", amod);

        Ok(())
    }
}
