use super::{
    slice::{Region, Slice},
    val::Val,
};
use crate::human_powered_vm::error::{Error, Result};
use pentagwam::{
    cell::{Cell, Functor},
    defs::{CellRef, Sym},
};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValTy {
    CellRef,
    Cell,
    Usize,
    I32,
    Symbol,
    Functor,
    TypeOf(String),
    Slice,
}

impl ValTy {
    pub fn default_val(&self) -> Val {
        match self {
            ValTy::CellRef => Val::CellRef(CellRef::new(0)),
            ValTy::Cell => Val::Cell(Cell::Nil),
            ValTy::Usize => Val::Usize(0),
            ValTy::I32 => Val::I32(0),
            ValTy::Symbol => Val::Symbol("".to_string()),
            ValTy::Functor => Val::Cell(Cell::Sig(Functor {
                sym: Sym::new(0),
                arity: 0,
            })),
            ValTy::TypeOf(_) => panic!("Can't create default value for `TypeOf(..)`"),
            ValTy::Slice => Val::Slice(Slice {
                region: Region::Mem,
                start: 0,
                len: 0,
            }),
        }
    }
}

impl fmt::Display for ValTy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValTy::CellRef => write!(f, "CellRef"),
            ValTy::Cell => write!(f, "Cell"),
            ValTy::Usize => write!(f, "Usize"),
            ValTy::I32 => write!(f, "I32"),
            ValTy::Symbol => write!(f, "Symbol"),
            ValTy::Functor => write!(f, "Functor"),
            ValTy::TypeOf(field) => write!(f, "TypeOf({field})"),
            ValTy::Slice => write!(f, "Slice"),
        }
    }
}

impl FromStr for ValTy {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "CellRef" => Ok(ValTy::CellRef),
            "AnyCellVal" => Ok(ValTy::Cell),
            "Usize" => Ok(ValTy::Usize),
            "I32" => Ok(ValTy::I32),
            "Symbol" => Ok(ValTy::Symbol),
            "Functor" => Ok(ValTy::Functor),
            _ if s.starts_with("TypeOf(") && s.ends_with(')') => {
                let field_name = &s["TypeOf(".len()..s.len() - 1];
                Ok(ValTy::TypeOf(field_name.to_string()))
            }
            _ => Err(Error::ParseTypeError(s.to_string())),
        }
    }
}
