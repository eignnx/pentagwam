use crate::defs::{Idx, Sym};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Cell {
    /// A reference (usually represents a variable).
    Ref(Idx),
    /// A record with an index to its functor.
    Rcd(Idx),
    /// An integer.
    Int(i32),
    /// A symbol.
    Sym(Sym),
    /// The identifier of a functor.
    Sig(Functor),
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cell::Ref(idx) => write!(f, "Ref({})", idx),
            Cell::Rcd(idx) => write!(f, "Rcd({})", idx),
            Cell::Int(i) => write!(f, "Int({})", i),
            Cell::Sym(sym) => write!(f, "Sym({})", sym),
            Cell::Sig(functor) => write!(f, "Sig({})", functor),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Functor {
    pub sym: Sym,
    pub arity: u8,
}

impl std::fmt::Display for Functor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}/{}", self.sym, self.arity)
    }
}
