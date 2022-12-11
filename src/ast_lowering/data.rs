use std::cmp::{max, min};

use indexmap::{indexmap, IndexMap};
use inkwellkit::{
    get_ctx,
    types::{FloatType, IntType},
};
use m6coll::KVEntry as Entry;
use m6lexerkit::{str2sym, sym2str, Symbol, lazy_static::lazy_static, Token};

use super::MIR;
use crate::{
    name_mangling::mangling,
    parser::SyntaxType as ST, core::load_core_exp,
};


////////////////////////////////////////////////////////////////////////////////
//// Constant

lazy_static! {
    /// Exported Symbol Set
    pub static ref ESS: ExtSymSet = {
        let core = load_core_exp();

        ExtSymSet {
            mods: vec![core]
        }
    };
}


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


#[derive(Clone, Debug)]
pub struct AnExtFnDec {
    // idt: Token,  // Identifier Token
    // body_idx: Option<usize>,
    pub name: Symbol,
    pub params: Vec<(Symbol, AType)>,
    pub ret: AType,
    pub sign_name: Symbol,
}


pub(crate) struct AMod {
    pub(crate) name: Symbol,
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
    Str,       // C string
    // Char,  // u32
    OpaqueStruct(Symbol), // opaque struct pointer type
}


pub struct AFnDec {
    pub idt: Token,  // Identifier Token
    // body_idx: Option<usize>,
    pub name: Symbol,
    pub params: Vec<AParamPat>,
    pub ret: AType,
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
        sign_name: Symbol,
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

impl ExtSymSet {
    pub(crate) fn find_func(
        &self,
        name: &str,
        atys: &[AType],
    ) -> Option<&AnExtFnDec> {
        let fullname = mangling(str2sym(name), atys);

        self.find_func_by_name(fullname)
    }

    pub(crate) fn find_func_by_name(
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

    pub(crate) fn find_unique_func(
        &self,
        name: &str,
    ) -> Result<&AnExtFnDec, Vec<&AnExtFnDec>> {

        let fullname = str2sym(name);
        let mut res = vec![];

        for amod in self.mods.iter() {
            if let Some(afndec) = amod.in_mod_exp_find(fullname) {
                res.push(afndec);
            }
        }

        if res.is_empty() || res.len() > 1 {
            Err(res)
        }
        else {
            Ok(res[0])
        }

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
                call_fn: self.name,
                args: args.into_iter().cloned().collect(),
                sign_name: self.sign_name,
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
            afns: indexmap! {},
            allocs: indexmap! {},
            scopes: vec![AScope::default()], // push Root Scope
        }
    }

    pub(crate) fn in_mod_find_funsym(
        &self,
        fullname: Symbol,
    ) -> Option<&AFnDec> {
        self.afns.get(&fullname)
    }

    pub(crate) fn export(&self) -> AModExp {
        let afns = self.afns
            .iter()
            .map(|(k, v)| (*k, v.as_ext_fn_dec()))
            .collect();

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
            name: self.name,
            params: self
                .params
                .iter()
                .cloned()
                .map(|x| (x.formal, x.ty))
                .collect(),
            ret: self.ret.clone(),
            sign_name: self.name,
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
    pub(crate) fn efn_call(efn_dec: &AnExtFnDec, params: Vec<Symbol>) -> Self {
        Self {
            ty: efn_dec.ret.clone(),
            val: AVal::FnCall {
                call_fn: efn_dec.name,
                args: params,
                sign_name: efn_dec.sign_name,
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
                            (APriType::Str, APriType::Str) => aty_str(),
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
#[allow(unused)]
pub(crate) fn aty_opaque_struct(s: &str) -> AType {
    AType::Pri(APriType::OpaqueStruct(str2sym(s)))
}
pub(crate) const fn aty_arr_int() -> AType {
    AType::Arr(APriType::Int(-4), 1)
}
pub(crate) const fn aty_arr_float() -> AType {
    AType::Arr(APriType::Float(8), 1)
}
pub(crate) const fn aty_arr_str() -> AType {
    AType::Arr(APriType::Str, 1)
}
