use derive_more::From;
use pentagwam::{
    cell::{Cell, Functor},
    defs::CellRef,
    mem::{DisplayViaMem, Mem},
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};

use super::{
    rval::SLICE_IDX_LEN_SEP,
    slice::Region,
    valty::{CellTy, ValTy},
};
use crate::human_powered_vm::error::{Error, Result};

#[derive(Debug, From, Clone, Serialize, Deserialize)]
pub enum Val {
    #[from]
    CellRef(CellRef),
    Usize(usize),
    I32(i32),
    Symbol(String),
    Cell(Cell),
    Slice {
        region: Region,
        start: usize,
        len: usize,
    },
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
            Val::Slice { region, start, len } => {
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
            Val::Cell(cell) => match cell {
                Cell::Ref(..) => ValTy::Cell(Some(CellTy::Ref)),
                Cell::Rcd(..) => ValTy::Cell(Some(CellTy::Rcd)),
                Cell::Int(..) => ValTy::Cell(Some(CellTy::Int)),
                Cell::Sym(..) => ValTy::Cell(Some(CellTy::Sym)),
                Cell::Sig(..) => ValTy::Cell(Some(CellTy::Sig)),
                Cell::Lst(..) => ValTy::Cell(Some(CellTy::Lst)),
                Cell::Nil => ValTy::Cell(Some(CellTy::Nil)),
            },
            Val::Slice { .. } => ValTy::Slice,
        }
    }

    /// Will convert some `Cell` values to `CelRef`s also.
    pub fn try_as_cell_ref(&self, mem: &Mem) -> Result<CellRef> {
        self.try_convert(ValTy::CellRef, mem).map(|val| match val {
            Val::CellRef(cell_ref) => cell_ref,
            _ => unreachable!(),
        })
    }

    pub fn try_as_i32(&self, mem: &Mem) -> Result<i32> {
        self.try_convert(ValTy::I32, mem).map(|val| match val {
            Val::I32(i) => i,
            _ => unreachable!(),
        })
    }

    pub fn try_as_usize(&self, mem: &Mem) -> Result<usize> {
        self.try_convert(ValTy::Usize, mem).map(|val| match val {
            Val::Usize(u) => u,
            _ => unreachable!(),
        })
    }

    pub fn try_as_cell(&self, mem: &Mem) -> Result<Cell> {
        self.try_convert(ValTy::Cell(None), mem)
            .map(|val| match val {
                Val::Cell(cell) => cell,
                _ => unreachable!(),
            })
    }

    pub fn try_as_symbol<'a>(&'a self, mem: &'a Mem) -> Result<Cow<'a, str>> {
        self.try_convert(ValTy::Symbol, mem).map(|val| match val {
            Val::Symbol(s) => Cow::Owned(s),
            _ => unreachable!(),
        })
    }

    pub fn try_as_any_int(&self, mem: &Mem) -> Result<i64> {
        self.try_convert(ValTy::I32, mem).map(|val| match val {
            Val::I32(i) => i as i64,
            _ => unreachable!(),
        })
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
            Val::Slice { region, start, len } => {
                write!(f, "{region}[{start}{SLICE_IDX_LEN_SEP}{len}]")
            }
        }
    }
}

impl Val {
    /// Knows about the HPVM's (rather relaxed) type system, and it's (rather
    /// forgiving) conversion rules.
    ///
    /// Prefer to use this method (or even better: the more specific ones like
    /// `Val::try_as_*`) instead of matching directly on the `Val` enum.
    pub fn try_convert(&self, ty: ValTy, mem: &Mem) -> Result<Self> {
        match self {
            Val::CellRef(r) | Val::Cell(Cell::Ref(r)) => match ty {
                ValTy::CellRef => Ok(Val::CellRef(*r)),
                ValTy::Cell(None) | ValTy::Cell(Some(CellTy::Ref)) => Ok(Val::Cell(Cell::Ref(*r))),
                // NOTE: These variants are spelled out so that adding a new
                // variant results in a compilation error until this code has
                // been reviewed and updated.
                ValTy::Cell(Some(CellTy::Int))
                | ValTy::Cell(Some(CellTy::Nil))
                | ValTy::Cell(Some(CellTy::Lst))
                | ValTy::Cell(Some(CellTy::Rcd))
                | ValTy::Cell(Some(CellTy::Sig))
                | ValTy::Cell(Some(CellTy::Sym))
                | ValTy::Usize
                | ValTy::I32
                | ValTy::Symbol
                | ValTy::Functor
                | ValTy::Slice => Err(Error::TypeError {
                    expected: ty.to_string(),
                    received: self.ty(),
                    expr: self.to_string(),
                }),
            },
            Val::Usize(u) => match ty {
                ValTy::Usize => Ok(self.clone()),
                ValTy::I32 => Ok(Val::I32(*u as i32)),
                ValTy::Cell(None) | ValTy::Cell(Some(CellTy::Int)) => {
                    Ok(Val::Cell(Cell::Int(*u as i32)))
                }
                ValTy::Cell(Some(CellTy::Nil))
                | ValTy::Cell(Some(CellTy::Lst))
                | ValTy::Cell(Some(CellTy::Ref))
                | ValTy::Cell(Some(CellTy::Rcd))
                | ValTy::Cell(Some(CellTy::Sig))
                | ValTy::Cell(Some(CellTy::Sym))
                | ValTy::CellRef
                | ValTy::Symbol
                | ValTy::Functor
                | ValTy::Slice => Err(Error::TypeError {
                    expected: ty.to_string(),
                    received: self.ty(),
                    expr: self.to_string(),
                }),
            },
            Val::I32(i) => match ty {
                ValTy::I32 => Ok(self.clone()),
                ValTy::Cell(None) | ValTy::Cell(Some(CellTy::Int)) => Ok(Val::Cell(Cell::Int(*i))),
                ValTy::Usize
                | ValTy::Cell(Some(CellTy::Nil))
                | ValTy::Cell(Some(CellTy::Lst))
                | ValTy::Cell(Some(CellTy::Ref))
                | ValTy::Cell(Some(CellTy::Rcd))
                | ValTy::Cell(Some(CellTy::Sig))
                | ValTy::Cell(Some(CellTy::Sym))
                | ValTy::CellRef
                | ValTy::Symbol
                | ValTy::Functor
                | ValTy::Slice => Err(Error::TypeError {
                    expected: ty.to_string(),
                    received: self.ty(),
                    expr: self.to_string(),
                }),
            },
            Val::Symbol(s) => match ty {
                ValTy::Symbol => Ok(self.clone()),
                ValTy::Cell(None) | ValTy::Cell(Some(CellTy::Sym)) => {
                    Ok(Val::Cell(Cell::Sym(mem.intern_sym(s))))
                }
                ValTy::Cell(Some(CellTy::Int))
                | ValTy::Cell(Some(CellTy::Nil))
                | ValTy::Cell(Some(CellTy::Lst))
                | ValTy::Cell(Some(CellTy::Ref))
                | ValTy::Cell(Some(CellTy::Rcd))
                | ValTy::Cell(Some(CellTy::Sig))
                | ValTy::CellRef
                | ValTy::Usize
                | ValTy::I32
                | ValTy::Functor
                | ValTy::Slice => Err(Error::TypeError {
                    expected: ty.to_string(),
                    received: self.ty(),
                    expr: self.to_string(),
                }),
            },
            Val::Cell(Cell::Nil) => match ty {
                ValTy::Cell(None) | ValTy::Cell(Some(CellTy::Nil)) => Ok(self.clone()),
                _ => Err(Error::TypeError {
                    expected: ty.to_string(),
                    received: self.ty(),
                    expr: self.to_string(),
                }),
            },
            Val::Cell(Cell::Int(i)) => match ty {
                ValTy::Cell(None) | ValTy::Cell(Some(CellTy::Int)) => Ok(self.clone()),
                ValTy::I32 => Ok(Val::I32(*i)),
                _ => Err(Error::TypeError {
                    expected: ty.to_string(),
                    received: self.ty(),
                    expr: self.to_string(),
                }),
            },
            Val::Cell(Cell::Lst(r)) => match ty {
                ValTy::Cell(None) | ValTy::Cell(Some(CellTy::Lst)) => Ok(self.clone()),
                ValTy::CellRef => Ok(Val::CellRef(*r)),
                _ => Err(Error::TypeError {
                    expected: ty.to_string(),
                    received: self.ty(),
                    expr: self.to_string(),
                }),
            },
            Val::Cell(Cell::Rcd(r)) => match ty {
                ValTy::Cell(None) | ValTy::Cell(Some(CellTy::Rcd)) => Ok(self.clone()),
                ValTy::CellRef => Ok(Val::CellRef(*r)),
                _ => Err(Error::TypeError {
                    expected: ty.to_string(),
                    received: self.ty(),
                    expr: self.to_string(),
                }),
            },
            Val::Cell(Cell::Sig(_)) => match ty {
                ValTy::Cell(None) | ValTy::Cell(Some(CellTy::Sig)) | ValTy::Functor => {
                    Ok(self.clone())
                }
                _ => Err(Error::TypeError {
                    expected: ty.to_string(),
                    received: self.ty(),
                    expr: self.to_string(),
                }),
            },
            Val::Cell(Cell::Sym(s)) => match ty {
                ValTy::Cell(None) | ValTy::Cell(Some(CellTy::Sym)) => Ok(self.clone()),
                ValTy::Symbol => Ok(Val::Symbol(s.resolve(mem).to_string())),
                _ => Err(Error::TypeError {
                    expected: ty.to_string(),
                    received: self.ty(),
                    expr: self.to_string(),
                }),
            },
            Val::Slice { .. } => match ty {
                ValTy::Slice => Ok(self.clone()),
                _ => Err(Error::TypeError {
                    expected: ty.to_string(),
                    received: self.ty(),
                    expr: self.to_string(),
                }),
            },
        }
    }
}
