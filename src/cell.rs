use crate::defs::Idx;

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

#[derive(Debug, Clone, Copy)]
pub struct Functor {
    pub id: usize,
    pub arity: u8,
}
