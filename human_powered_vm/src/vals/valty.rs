use super::{slice::Region, val::Val};
use crate::human_powered_vm::error::{Error, Result};
use pentagwam::{cell::Cell, defs::CellRef, mem::Mem};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValTy {
    CellRef,
    Cell(Option<CellTy>),
    Usize,
    I32,
    Symbol,
    Functor,
    Slice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellTy {
    Sig,
    Int,
    Sym,
    Ref,
    Rcd,
    Lst,
    Nil,
}

impl ValTy {
    pub fn default_val(&self, mem: &Mem) -> Val {
        match self {
            ValTy::CellRef => Val::CellRef(CellRef::new(0)),
            ValTy::Cell(None) => Val::Cell(Cell::Nil),
            ValTy::Cell(Some(cell_ty)) => match cell_ty {
                CellTy::Lst => Val::Cell(Cell::Lst(CellRef::new(0))),
                CellTy::Nil => Val::Cell(Cell::Nil),
                CellTy::Sig => Val::Cell(Cell::Sig(mem.intern_functor("<default>", 0))),
                CellTy::Int => Val::Cell(Cell::Int(0)),
                CellTy::Sym => Val::Cell(Cell::Sym(mem.intern_sym("<default>"))),
                CellTy::Ref => Val::Cell(Cell::Ref(CellRef::new(0))),
                CellTy::Rcd => Val::Cell(Cell::Rcd(CellRef::new(0))),
            },
            ValTy::Usize => Val::Usize(0),
            ValTy::I32 => Val::I32(0),
            ValTy::Symbol => Val::Symbol("<default>".to_string()),
            ValTy::Functor => Val::Functor {
                sym: "<default>".to_string(),
                arity: 0,
            },
            ValTy::Slice => Val::Slice {
                region: Region::Mem,
                start: 0,
                len: 0,
            },
        }
    }
}

impl fmt::Display for ValTy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValTy::CellRef => write!(f, "CellRef"),
            ValTy::Cell(None) => write!(f, "Cell"),
            ValTy::Cell(Some(cell_ty)) => write!(f, "Cell({:?})", cell_ty),
            ValTy::Usize => write!(f, "Usize"),
            ValTy::I32 => write!(f, "I32"),
            ValTy::Symbol => write!(f, "Symbol"),
            ValTy::Functor => write!(f, "Functor"),
            ValTy::Slice => write!(f, "Slice"),
        }
    }
}

impl FromStr for ValTy {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "CellRef" => Ok(ValTy::CellRef),
            "Cell(Sig)" => Ok(ValTy::Cell(Some(CellTy::Sig))),
            "Cell(Int)" => Ok(ValTy::Cell(Some(CellTy::Int))),
            "Cell(Sym)" => Ok(ValTy::Cell(Some(CellTy::Sym))),
            "Cell(Ref)" => Ok(ValTy::Cell(Some(CellTy::Ref))),
            "Cell(Rcd)" => Ok(ValTy::Cell(Some(CellTy::Rcd))),
            "Cell(Lst)" => Ok(ValTy::Cell(Some(CellTy::Lst))),
            "Cell(Nil)" => Ok(ValTy::Cell(Some(CellTy::Nil))),
            "Cell" => Ok(ValTy::Cell(None)),
            "Usize" => Ok(ValTy::Usize),
            "I32" => Ok(ValTy::I32),
            "Symbol" => Ok(ValTy::Symbol),
            "Functor" => Ok(ValTy::Functor),
            "Slice" => Ok(ValTy::Slice),
            _ => Err(Error::ParseTypeError(s.to_string())),
        }
    }
}
