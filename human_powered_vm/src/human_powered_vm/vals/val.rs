use super::slice::Slice;
use derive_more::From;
use pentagwam::{
    cell::{Cell, Functor},
    defs::CellRef,
    mem::{DisplayViaMem, Mem},
};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::{rval::SLICE_IDX_LEN_SEP, valty::ValTy};
use crate::human_powered_vm::error::{Error, Result};

#[derive(Debug, From, Clone, Serialize, Deserialize)]
pub enum Val {
    #[from]
    CellRef(CellRef),
    Usize(usize),
    I32(i32),
    Symbol(String),
    Cell(Cell),
    Slice(Slice<usize>),
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
            Val::Symbol(s) => write!(f, ":{s}"),
            Val::Cell(cell) => write!(f, "{cell:?}"),
            Val::Slice(Slice { region, start, len }) => {
                write!(f, "{region}[{start}{SLICE_IDX_LEN_SEP}{len}]")
            }
        }
    }
}

impl Val {
    pub fn ty(&self) -> ValTy {
        match self {
            Val::CellRef(..) => ValTy::CellRef,
            Val::Usize(..) => ValTy::Usize,
            Val::I32(..) => ValTy::I32,
            Val::Symbol(..) => ValTy::Symbol,
            Val::Cell(..) => ValTy::Cell,
            Val::Slice { .. } => ValTy::Slice,
        }
    }

    pub fn try_as_cell_ref(&self) -> Result<CellRef> {
        match self {
            Val::CellRef(cell_ref) => Ok(*cell_ref),
            other => Err(Error::TypeError {
                expected: "CellRef".into(),
                received: other.ty(),
            }),
        }
    }

    pub fn try_as_cell_ref_like(&self) -> Result<CellRef> {
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

    pub fn try_as_i32(&self) -> Result<i32> {
        match self {
            Val::I32(i) => Ok(*i),
            other => Err(Error::TypeError {
                expected: "i32".into(),
                received: other.ty(),
            }),
        }
    }

    pub fn try_as_usize(&self) -> Result<usize> {
        match self {
            Val::Usize(u) => Ok(*u),
            other => Err(Error::TypeError {
                expected: "usize".into(),
                received: other.ty(),
            }),
        }
    }

    pub fn try_as_cell(&self) -> Result<Cell> {
        match self {
            Val::Cell(cell) => Ok(*cell),
            other => Err(Error::TypeError {
                expected: "Cell".into(),
                received: other.ty(),
            }),
        }
    }

    pub fn try_as_symbol(&self) -> Result<&str> {
        match self {
            Val::Symbol(s) => Ok(s),
            other => Err(Error::TypeError {
                expected: "Symbol".into(),
                received: other.ty(),
            }),
        }
    }

    pub fn try_as_any_int(&self) -> Result<i64> {
        match self {
            Val::I32(i) => Ok(*i as i64),
            Val::Usize(u) => Ok(*u as i64),
            other => Err(Error::TypeError {
                expected: "I32 or Usize".into(),
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
            Val::Symbol(s) => {
                if s.contains(|c: char| !c.is_alphanumeric() && c != '_')
                    || !s.starts_with(|c: char| c.is_alphabetic() || c == '_')
                {
                    write!(f, ":'{s}'")
                } else {
                    write!(f, ":{s}")
                }
            }
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
            Val::Slice(Slice { region, start, len }) => {
                write!(f, "{region}[{start}{SLICE_IDX_LEN_SEP}{len}]")
            }
        }
    }
}
