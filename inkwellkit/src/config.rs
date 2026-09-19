use std::{
    path::PathBuf,
};

use clap::{ArgEnum, PossibleValue};
use inkwell::OptimizationLevel;


///////////////////////////////////////////////////////////////////////////
//// Compiler Config

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum OptLv {
    Debug,
    Opt1,
    Opt2,
    Opt3
}

impl ArgEnum for OptLv {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            OptLv::Debug,
            OptLv::Opt1,
            OptLv::Opt2,
            OptLv::Opt3
        ]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::PossibleValue<'a>> {
        Some(match self {
            OptLv::Debug => PossibleValue::new("0"),
            OptLv::Opt1 => PossibleValue::new("1"),
            OptLv::Opt2 => PossibleValue::new("2"),
            OptLv::Opt3 => PossibleValue::new("3"),
        })
    }
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum TargetType {
    #[default]
    Bin,
    ReLoc,
    DyLib,
}

impl ArgEnum for TargetType {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::Bin,
            Self::ReLoc,
            Self::DyLib
        ]
    }

    fn to_possible_value<'a>(&self) -> Option<PossibleValue<'a>> {
        Some(match self {
            Self::Bin => PossibleValue::new("bin"),
            Self::ReLoc => PossibleValue::new("lib"),
            Self::DyLib => PossibleValue::new("dylib"),
        })
    }
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum EmitType {
    LLVMIR,
    Asm,
    #[default]
    Obj,
}

impl ArgEnum for EmitType {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::LLVMIR,
            Self::Asm,
            Self::Obj
        ]
    }

    fn to_possible_value<'a>(&self) -> Option<PossibleValue<'a>> {
        Some(PossibleValue::new(match self {
            Self::LLVMIR => "llvm-ir",
            Self::Asm => "asm",
            Self::Obj => "obj",
        }))
    }
}


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum VerboseLv {
    V1,
    V2,
    V3,
}

impl ArgEnum for VerboseLv {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::V1,
            Self::V2,
            Self::V3
        ]
    }

    fn to_possible_value<'a>(&self) -> Option<PossibleValue<'a>> {
        Some(PossibleValue::new(match self {
            Self::V1 => "v1",
            Self::V2 => "v2",
            Self::V3 => "v3",
        }))
    }
}

impl From<usize> for VerboseLv {
    fn from(i: usize) -> Self {
        match i {
            // 0 => Self::V0,
            1 => Self::V1,
            2 => Self::V2,
            _ => Self::V1,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PrintTy {
    StdErr,
    File(PathBuf),
}

impl PrintTy {
    pub fn get_path(&self) -> Option<PathBuf> {
        if let Self::File(path) = self {
            Some(path.clone())
        } else {
            None
        }
    }
}

pub const fn usize_len() -> usize {
    if cfg!(target_pointer_width = "64") {
        8
    } else {
        4
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CompilerConfig {
    pub optlv: OptLv,
    pub target_type: TargetType,
    pub emit_type: EmitType,
    pub print_type: PrintTy,
}


impl Into<OptimizationLevel> for OptLv {
    fn into(self) -> OptimizationLevel {
        match &self {
            Self::Debug => OptimizationLevel::None,
            Self::Opt1 => OptimizationLevel::Less,
            Self::Opt2 => OptimizationLevel::Default,
            Self::Opt3 => OptimizationLevel::Aggressive,
        }
    }
}
