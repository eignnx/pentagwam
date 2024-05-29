//! Custom formatting for the `Val` type. Wraps a `Val` in a `ValFmt` and
//! includes a reference to the `Mem` for formatting `CellRef`s.

use super::*;

pub struct ValFmt<'a> {
    val: &'a Val,
    mem: &'a Mem,
}

impl Val {
    pub fn display<'a>(&'a self, mem: &'a Mem) -> ValFmt<'a> {
        ValFmt { val: self, mem }
    }
}

impl<'a> std::fmt::Display for ValFmt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.val {
            Val::CellRef(cell_ref) => write!(f, "{cell_ref}"),
            Val::Usize(u) => write!(f, "{u}"),
            Val::I32(i) => write!(f, "{i:+}"),
            Val::Cell(Cell::Int(i)) => write!(f, "Int({i:+})"),
            Val::Cell(Cell::Sig(Functor { sym, arity })) => {
                let sym = sym.resolve(self.mem);
                if sym.contains(|c: char| !c.is_alphanumeric() && c != '_')
                    || !sym.starts_with(|c: char| c.is_alphabetic() || c == '_')
                {
                    write!(f, "Sig('{sym}'/{arity})")
                } else {
                    write!(f, "Sig({sym}/{arity})")
                }
            }
            Val::Cell(Cell::Sym(sym)) => {
                let sym = sym.resolve(self.mem);
                if sym.contains(|c: char| !c.is_alphanumeric() && c != '_')
                    || !sym.starts_with(|c: char| c.is_alphabetic() || c == '_')
                {
                    write!(f, "Sym('{sym}')")
                } else {
                    write!(f, "Sym({sym})")
                }
            }
            Val::Cell(Cell::Ref(cell_ref)) => {
                let name = self.mem.human_readable_var_name(*cell_ref);
                write!(f, "Ref({name}{cell_ref})")
            }
            Val::Cell(Cell::Rcd(cell_ref)) => write!(f, "Rcd({cell_ref})"),
            Val::Cell(Cell::Lst(cell_ref)) => write!(f, "Lst({cell_ref})"),
            Val::Cell(Cell::Nil) => write!(f, "Nil"),
        }
    }
}

pub struct RValFmt<'a> {
    rval: &'a RVal,
    mem: &'a Mem,
}

impl RVal {
    pub fn display<'a>(&'a self, mem: &'a Mem) -> RValFmt<'a> {
        RValFmt { rval: self, mem }
    }
}

impl std::fmt::Display for RValFmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.rval {
            RVal::Deref(inner) => write!(f, "*{}", inner.display(self.mem)),
            RVal::CellRef(r) => write!(f, "{r}"),
            RVal::Usize(u) => write!(f, "{u}"),
            RVal::I32(i) => write!(f, "{i:+}"),
            RVal::Field(field) => write!(f, "{field}"),
            RVal::InstrPtr => write!(f, "instr_ptr"),
            RVal::Cell(cell) => write!(f, "{}", cell.display(self.mem)),
        }
    }
}

pub struct FmtCellVal<'a, T> {
    cell: &'a CellVal<T>,
    mem: &'a Mem,
}

impl<T> CellVal<T> {
    pub fn display<'a>(&'a self, mem: &'a Mem) -> FmtCellVal<'a, T> {
        FmtCellVal { cell: self, mem }
    }
}

impl<T: std::fmt::Display> std::fmt::Display for FmtCellVal<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.cell {
            CellVal::Int(i) => write!(f, "{i:+}"),
            CellVal::Sig { fname, arity } => {
                if fname.contains(|c: char| !c.is_alphanumeric() && c != '_')
                    || !fname.starts_with(|c: char| c.is_alphabetic() || c == '_')
                {
                    write!(f, "Sig('{fname}'/{arity})")
                } else {
                    write!(f, "Sig({fname}/{arity})")
                }
            }
            CellVal::Sym(sym) => {
                if sym.contains(|c: char| !c.is_alphanumeric() && c != '_')
                    || !sym.starts_with(|c: char| c.is_alphabetic() || c == '_')
                {
                    write!(f, "Sym('{sym}')")
                } else {
                    write!(f, "Sym({sym})")
                }
            }
            CellVal::Ref(cell_ref) => write!(f, "Ref({cell_ref})"),
            CellVal::Rcd(cell_ref) => write!(f, "Rcd({cell_ref})"),
            CellVal::Lst(cell_ref) => write!(f, "Lst({cell_ref})"),
            CellVal::Nil => write!(f, "Nil"),
        }
    }
}

pub struct LValFmt<'a> {
    lval: &'a LVal,
    mem: &'a Mem,
}

impl LVal {
    pub fn display<'a>(&'a self, mem: &'a Mem) -> LValFmt<'a> {
        LValFmt { lval: self, mem }
    }
}

impl std::fmt::Display for LValFmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.lval {
            LVal::CellRef(cell_ref) => write!(f, "{cell_ref}"),
            LVal::Field(field) => write!(f, "{field}"),
            LVal::InstrPtr => write!(f, "InstrPtr"),
            LVal::Deref(rval) => write!(
                f,
                "*{}",
                RValFmt {
                    rval,
                    mem: self.mem
                }
            ),
        }
    }
}
