use derive_more::From;
use pentagwam::{
    cell::{Cell, Functor},
    defs::CellRef,
    mem::{DisplayViaMem, Mem},
};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::human_powered_vm::error::{Error, Result};

use super::valty::ValTy;

#[derive(Debug, From, Clone, Serialize, Deserialize)]
pub enum Val {
    #[from]
    CellRef(CellRef),
    Usize(usize),
    I32(i32),
    Cell(Cell),
}

impl Default for Val {
    fn default() -> Self {
        Self::Cell(Cell::Nil)
    }
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Val::CellRef(cell_ref) => write!(f, "{cell_ref}"),
            Val::Usize(u) => write!(f, "{u}"),
            Val::I32(i) => write!(f, "{i:+}"),
            Val::Cell(cell) => write!(f, "{cell:?}"),
        }
    }
}

impl Val {
    pub fn ty(&self) -> ValTy {
        match self {
            Val::CellRef(_) => ValTy::CellRef,
            Val::Usize(_) => ValTy::Usize,
            Val::I32(_) => ValTy::I32,
            Val::Cell(_) => ValTy::AnyCellVal,
        }
    }

    pub fn expect_cell_ref(&self) -> Result<CellRef> {
        match self {
            Val::CellRef(cell_ref) => Ok(*cell_ref),
            other => Err(Error::TypeError {
                expected: "CellRef".into(),
                received: other.ty(),
            }),
        }
    }

    pub fn expect_cell_ref_like(&self) -> Result<CellRef> {
        match self {
            Val::CellRef(cell_ref)
            | Val::Cell(Cell::Ref(cell_ref))
            | Val::Cell(Cell::Rcd(cell_ref))
            | Val::Cell(Cell::Lst(cell_ref)) => Ok(*cell_ref),
            other => Err(Error::TypeError {
                expected: "CellRef, Ref, Rcd, or Lst".into(),
                received: other.ty(),
            }),
        }
    }

    pub fn expect_i32(&self) -> Result<i32> {
        match self {
            Val::I32(i) => Ok(*i),
            other => Err(Error::TypeError {
                expected: "i32".into(),
                received: other.ty(),
            }),
        }
    }

    pub fn expect_usize(&self) -> Result<usize> {
        match self {
            Val::Usize(u) => Ok(*u),
            other => Err(Error::TypeError {
                expected: "usize".into(),
                received: other.ty(),
            }),
        }
    }

    pub fn expect_cell(&self) -> Result<Cell> {
        match self {
            Val::Cell(cell) => Ok(*cell),
            other => Err(Error::TypeError {
                expected: "Cell".into(),
                received: other.ty(),
            }),
        }
    }
}

impl DisplayViaMem for Val {
    fn display_via_mem(&self, f: &mut fmt::Formatter<'_>, mem: &Mem) -> fmt::Result {
        match self {
            Val::CellRef(cell_ref) => write!(f, "{cell_ref}"),
            Val::Usize(u) => write!(f, "{u}"),
            Val::I32(i) => write!(f, "{i:+}"),
            Val::Cell(Cell::Int(i)) => write!(f, "Int({i:+})"),
            Val::Cell(Cell::Sig(Functor { sym, arity })) => {
                let sym = sym.resolve(mem);
                if sym.contains(|c: char| !c.is_alphanumeric() && c != '_')
                    || !sym.starts_with(|c: char| c.is_alphabetic() || c == '_')
                {
                    write!(f, "Sig('{sym}'/{arity})")
                } else {
                    write!(f, "Sig({sym}/{arity})")
                }
            }
            Val::Cell(Cell::Sym(sym)) => {
                let sym = sym.resolve(mem);
                if sym.contains(|c: char| !c.is_alphanumeric() && c != '_')
                    || !sym.starts_with(|c: char| c.is_alphabetic() || c == '_')
                {
                    write!(f, "Sym('{sym}')")
                } else {
                    write!(f, "Sym({sym})")
                }
            }
            Val::Cell(Cell::Ref(cell_ref)) => {
                let name = mem.human_readable_var_name(*cell_ref);
                write!(f, "Ref({name}{cell_ref})")
            }
            Val::Cell(Cell::Rcd(cell_ref)) => write!(f, "Rcd({cell_ref})"),
            Val::Cell(Cell::Lst(cell_ref)) => write!(f, "Lst({cell_ref})"),
            Val::Cell(Cell::Nil) => write!(f, "Nil"),
        }
    }
}
