use crate::defs::{Idx, Sym};

pub union Cell {
    pub(crate) functor: Functor,
    pub(crate) tagged: CellVal,
}

impl From<CellVal> for Cell {
    fn from(tagged: CellVal) -> Self {
        Self { tagged }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CellVal {
    /// A reference (usually represents a variable).
    Ref(Idx),
    /// A record with an index to its functor.
    Rcd(Idx),
}

impl Cell {
    pub fn r#ref(idx: impl Into<Idx>) -> Self {
        CellVal::Ref(idx.into()).into()
    }

    pub fn rcd(idx: impl Into<Idx>) -> Self {
        CellVal::Rcd(idx.into()).into()
    }

    pub fn functor(functor: Functor) -> Self {
        Self { functor }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Functor {
    pub sym: Sym,
    pub arity: u8,
}
