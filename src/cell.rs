use serde::{Deserialize, Serialize};

use crate::defs::{CellRef, Sym};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Functor {
    pub sym: Sym,
    pub arity: u8,
}

impl std::fmt::Debug for Functor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}/{}", self.sym, self.arity)
    }
}

impl std::fmt::Display for Functor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
