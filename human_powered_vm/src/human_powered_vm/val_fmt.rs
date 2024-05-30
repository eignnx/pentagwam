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
            RVal::Index(base, offset) => write!(
                f,
                "{}[{}]",
                base.display(self.mem),
                offset.display(self.mem),
            ),
            RVal::CellRef(r) => write!(f, "{r}"),
            RVal::Usize(u) => write!(f, "{u}"),
            RVal::I32(i) => write!(f, "{i:+}"),
            RVal::Field(field) => write!(f, "self.{field}"),
            RVal::TmpVar(name) => write!(f, ".{name}"),
            RVal::InstrPtr => write!(f, "instr_ptr"),
            RVal::Cell(cell) => write!(f, "{}", cell.display()),
        }
    }
}
