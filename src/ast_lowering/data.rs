use std::{cmp::{max, min}, fmt::Debug};

use indexmap::{indexmap, IndexMap};
use inkwellkit::{
    get_ctx,
    types::{FloatType, IntType},
};
use m6coll::KVEntry as Entry;
use m6lexerkit::{str2sym, sym2str, Symbol, Token};

use super::MIR;
use crate::parser::SyntaxType as ST;


////////////////////////////////////////////////////////////////////////////////
//// Constant


////////////////////////////////////////////////////////////////////////////////
//// Structure


/// Exported Symbol Set
pub struct ExtSymSet {
    pub mods: Vec<AModExp>,
}


/// An Exported Mod
pub struct AModExp {
    pub afns: IndexMap<Symbol, AnExtFnDec>,
}


#[derive(Clone)]
pub struct AnExtFnDec {
    pub attrs: A3ttrs,
    pub full_name: Symbol,
    pub params: Vec<AParamPat>,
    pub ret: AType,
    pub symbol_name: Symbol,
}


pub struct AFnDec {
    pub idt: Token, // Identifier Token
    pub attrs: A3ttrs,
    // body_idx: Option<usize>,
    pub name: Symbol,
    pub params: Vec<AParamPat>,
    pub ret: AType,
}


/// An Annotated Atrrs (Collection)
#[derive(Debug, Clone)]
pub struct A3ttrs(pub IndexMap<A3ttrName, A3ttrVal>);


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum A3ttrName {
    NoMangle,
    VarArg,
}


#[derive(Debug, Clone)]
pub enum A3ttrVal {
    Empty,
}


pub struct AMod {
    pub(crate) name: Symbol,
    /// External Declare
    pub(crate) efns: IndexMap<Symbol, AnExtFnDec>,
    /// Local Definition
    pub(crate) afns: IndexMap<Symbol, AFnDec>,
    pub(crate) allocs: IndexMap<Symbol, AFnAlloc>,
    pub(crate) scopes: Vec<AScope>, // Start from Root Scope
}


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AType {
    Pri(APriType),
    Arr(APriType, u8), // Normal Array (usize index)
    #[allow(unused)]
    AA(Vec<APriType>), // Associative Array (str index)
    Void,
    PH, // Phantom or Place Holder anyway, used for multiple diagnosis
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum APriType {
    Float(u8), // float64
    Int(i8),   // i32
    Ptr,       // C void*
    // Char,  // u32
    OpaqueStruct(Symbol), // opaque struct pointer type
}


pub(crate) type AFnAlloc = IndexMap<(Symbol, usize), AType>;


#[derive(Debug, Clone)]
pub(crate) struct ASymDef {
    pub(crate) name: Symbol,
    pub(crate) ty: AType,
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
        scope_idx: usize,
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
        ty: AType,
    },
    ConstAlias(ConstVal),
    Var(Symbol, usize), // symname, tagid : get var value
    Assign(Symbol, usize, Symbol), //  namesym, tagid, valsym : set var value
    Break,
    Continue,
    Return(Option<Symbol>),
    PH,
}


#[derive(Debug, Clone)]
pub(crate) struct AVar {
    pub(crate) ty: AType,
    pub(crate) val: AVal, // MIR usize
}


#[derive(Default)]
pub(crate) struct AScope {
    pub(crate) paren: Option<usize>,
    /// val: tagid, ty
    pub(crate) explicit_bindings: Vec<Entry<Symbol, (usize, AType)>>,
    pub(crate) implicit_bindings: IndexMap<Symbol, usize>,
    pub(crate) mirs: Vec<MIR>,
    pub(crate) ret: Option<AVar>,
}


#[derive(Debug, Clone)]
pub struct AParamPat {
    pub formal: Symbol,
    pub ty: AType,
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl Debug for AModExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for (k, v) in self.afns.iter() {
            writeln!(f, "{k:?} =>")?;
            writeln!(f, "{v:#?}\n")?;
        }

        Ok(())
    }
}


impl Debug for AnExtFnDec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        for (name, _val) in self.attrs.0.iter() {
            writeln!(f, "@{name:?}")?;
        }

        write!(f,
            "{:?}(",
             self.full_name,
        )?;

        for (i, param) in self.params.iter().enumerate() {
            write!(f,
                "{:?}: {}{}",
                 param.formal,
                 param.ty.ident_name(),
                 if i < self.params.len() - 1 { ", " } else { "" }
            )?;
        }

        write!(f, ")")?;

        if self.ret != AType::Void {
            write!(f, "-> {}", self.ret.ident_name())?;
        }

        Ok(())
    }
}


impl A3ttrs {
    // pub fn push_attr(&mut self, name: A3ttrName, val: A3ttrVal) -> Result<(), A3ttrVal> {
    //     match self.0.insert(name, val) {
    //         Some(oldval) => Err(oldval),
    //         None => Ok(()),
    //     }
    // }

    pub fn get_attr(&self, name: A3ttrName) -> Option<&A3ttrVal> {
        self.0.get(&name)
    }

    pub fn new() -> Self {
        A3ttrs(IndexMap::new())
    }

    pub fn has(&self, name: A3ttrName) -> bool {
        self.get_attr(name).is_some()
    }
}


impl ExtSymSet {
    pub fn find_func_by_name(
        &self,
        fullname: Symbol,
    ) -> Option<&AnExtFnDec> {
        for amod in self.mods.iter() {
            if let Some(afndec) = amod.in_mod_exp_find(fullname) {
                return Some(afndec);
            }
        }

        None
    }

    pub fn afns_iter(&self) -> impl Iterator<Item=&AnExtFnDec> {
        self.mods.iter().map(|amod| amod.afns.values()).flatten()
    }
}


impl AModExp {
    pub(crate) fn in_mod_exp_find(
        &self,
        fullname: Symbol,
    ) -> Option<&AnExtFnDec> {
        self.afns.get(&fullname)
    }
}


impl AnExtFnDec {
    pub(crate) fn fn_call_val(&self, args: &[Symbol]) -> AVar {
        AVar {
            ty: self.ret.clone(),
            val: AVal::FnCall {
                call_fn: self.full_name,
                args: args.into_iter().cloned().collect(),
            },
        }
    }
}


impl std::fmt::Debug for AMod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct AScopeVec<'a>(&'a Vec<AScope>);
        impl<'a> std::fmt::Debug for AScopeVec<'a> {
            fn fmt(
                &self,
                f: &mut std::fmt::Formatter<'_>,
            ) -> std::fmt::Result {
                for (i, ascope) in self.0.iter().enumerate() {
                    writeln!(
                        f,
                        "{} {} {}",
                        "-".repeat(35),
                        i,
                        "-".repeat(35)
                    )?;
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
    pub(crate) fn init(name: Symbol) -> Self {
        Self {
            name,
            efns: indexmap! {},
            afns: indexmap! {},
            allocs: indexmap! {},
            scopes: vec![AScope::default()], // push Root Scope
        }
    }

    /// Export both External Declare and Local Definition
    pub(crate) fn export(&self) -> AModExp {
        let afns = self
            .afns
            .iter()
            .map(|(k, v)| (*k, v.as_ext_fn_dec()));

        let efns = self
            .efns
            .iter()
            .map(|(k, v)| (*k, v.clone()));

        let afns = afns.chain(efns).collect();

        AModExp { afns }
    }
}


impl AScope {
    pub(crate) fn tmp_name(&self) -> Symbol {
        str2sym(&format!("!__tmp_{}", self.implicit_bindings.len()))
    }

    pub(crate) fn ret(&self) -> AVar {
        if let Some(ret) = self.ret.clone() {
            ret
        } else {
            AVar::void()
        }
    }

    pub(crate) fn in_scope_find_sym(
        &self,
        q: &Symbol,
    ) -> Option<(usize, AType)> {
        self.explicit_bindings
            .iter()
            .rev()
            .find(|Entry(sym, _mir_idx)| sym == q)
            .and_then(|Entry(_sym, (tagid, ty))| Some((*tagid, ty.clone())))
    }
}


impl std::fmt::Debug for AScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AScope")
            .field("paren", &self.paren)
            .field("explicit_bindings", &self.explicit_bindings)
            .field("implicit_bindings", &self.implicit_bindings)
            .field("mirs", &self.mirs)
            .field("ret", &self.ret)
            .finish()
    }
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

impl AFnDec {
    pub(crate) fn as_ext_fn_dec(&self) -> AnExtFnDec {
        AnExtFnDec {
            attrs: self.attrs.clone(),
            full_name: self.name,
            params: self.params.clone(),
            ret: self.ret.clone(),
            symbol_name: self.name,
        }
    }
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

    /// External function call
    pub(crate) fn efn_call(efn_dec: AnExtFnDec, params: Vec<Symbol>) -> Self {
        Self {
            ty: efn_dec.ret,
            val: AVal::FnCall {
                call_fn: efn_dec.full_name,
                args: params,
            },
        }
    }
}


impl Default for AVar {
    fn default() -> Self {
        Self::void()
    }
}


impl ASymDef {
    pub(crate) fn new(name: Symbol, ty: AType) -> Self {
        Self { name, ty }
    }

    pub(crate) fn undefined() -> Self {
        Self {
            name: str2sym(""),
            ty: AType::PH,
        }
    }
}


impl AType {
    pub(crate) fn lift_tys(op: ST, ty1: Self, ty2: Self) -> Result<Self, ()> {
        if ty1 == Self::PH || ty2 == Self::PH {
            return Ok(Self::PH);
        }

        match op {
            // It contains risk of overflow
            ST::add
            | ST::sub
            | ST::lt
            | ST::le
            | ST::gt
            | ST::ge
            | ST::eq
            | ST::neq => {
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
                            (APriType::Ptr, APriType::Ptr) => aty_str(),
                            _ => return Err(()),
                        })
                    }
                    _ => Err(()),
                }
            }
            _ => unreachable!("op: {:#?}", op),
        }
    }

    pub(crate) fn try_cast(&self, ty: &Self) -> Result<(), ()> {
        if self == ty {
            return Ok(());
        }

        match (self, ty) {
            (Self::Pri(prity1), Self::Pri(prity2)) => {
                Ok(match (prity1, prity2) {
                    (APriType::Float(_fmeta), APriType::Int(_imeta)) => (),
                    (APriType::Int(_imeta1), APriType::Int(_imeta2)) => (),
                    _ => return Err(()),
                })
            }
            _ => Err(()),
        }
    }
}



impl APriType {
    pub(crate) fn as_float_ty<'ctx>(&self) -> FloatType<'ctx> {
        let ctx = get_ctx();
        match self {
            Self::Float(i8) => match i8 {
                4 => ctx.f32_type(),
                8 => ctx.f64_type(),
                _ => unimplemented!(),
            },
            _ => unreachable!(),
        }
    }

    pub(crate) fn as_int_ty<'ctx>(&self) -> IntType<'ctx> {
        let ctx = get_ctx();
        match self {
            Self::Int(i8) => match i8 {
                4 => ctx.i32_type(),
                8 => ctx.i64_type(),
                _ => unimplemented!(),
            },
            _ => unreachable!(),
        }
    }
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

    #[allow(unused)]
    pub(crate) fn get_tagid(&self) -> Option<usize> {
        match self {
            AVal::Var(_, tagid) | AVal::Assign(_, tagid, _) => Some(*tagid),
            _ => None,
        }
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Function

pub(crate) const fn aty_str() -> AType {
    AType::Pri(APriType::Ptr)
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
#[allow(unused)]
pub(crate) fn aty_opaque_struct(s: &str) -> AType {
    AType::Pri(APriType::OpaqueStruct(str2sym(s)))
}
#[allow(unused)]
pub(crate) const fn aty_arr_int() -> AType {
    AType::Arr(APriType::Int(-4), 1)
}
#[allow(unused)]
pub(crate) const fn aty_arr_float() -> AType {
    AType::Arr(APriType::Float(8), 1)
}
#[allow(unused)]
pub(crate) const fn aty_arr_str() -> AType {
    AType::Arr(APriType::Ptr, 1)
}
