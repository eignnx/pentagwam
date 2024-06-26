use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    defs::{CellRef, Sym},
    mem::{DisplayViaMem, Mem},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u64)]
pub enum Cell {
    /// A reference (usually represents a variable).
    Ref(CellRef),

    /// A record with an index to its functor.
    Rcd(CellRef),

    /// An integer.
    Int(i32),

    /// A symbol.
    Sym(Sym),

    /// The identifier of a functor.
    Sig(Functor),

    /// A reference to a cons structure (a pair of cells `(car, cdr)`).
    /// ## Note
    /// If the ref points to `Nil`, the list contains an empty list as it's
    /// first element; it's `car` is `[]`.
    Lst(CellRef),

    /// The empty list.
    Nil,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Int(i32::MIN)
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cell::Ref(r) => write!(f, "Ref({r})"),
            Cell::Rcd(r) => write!(f, "Rcd({r})"),
            Cell::Int(i) => write!(f, "Int({i})"),
            Cell::Sym(s) => write!(f, "Sym({s})"),
            Cell::Sig(s) => write!(f, "Sig({s})"),
            Cell::Lst(r) => write!(f, "Lst({r})"),
            Cell::Nil => write!(f, "Nil"),
        }
    }
}

impl DisplayViaMem for Cell {
    fn display_via_mem(&self, f: &mut fmt::Formatter<'_>, mem: &Mem) -> fmt::Result {
        match self {
            Cell::Ref(r) => {
                let name = mem.human_readable_var_name(*r);
                if name.starts_with('_') {
                    write!(f, "Ref({name})")
                } else {
                    write!(f, "Ref({name}{r})")
                }
            }
            Cell::Rcd(r) => write!(f, "Rcd({})", r),
            Cell::Int(i) => write!(f, "Int({})", i),
            Cell::Sym(s) => write!(f, "Sym({})", mem.display(s)),
            Cell::Sig(s) => write!(f, "Sig({})", mem.display(s)),
            Cell::Lst(r) => write!(f, "Lst({})", r),
            Cell::Nil => write!(f, "Nil"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Functor<S = Sym> {
    pub sym: S,
    pub arity: u8,
}

impl<S: std::fmt::Debug> std::fmt::Debug for Functor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}/{}", self.sym, self.arity)
    }
}

impl<S: std::fmt::Display> std::fmt::Display for Functor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.sym, self.arity)
    }
}
