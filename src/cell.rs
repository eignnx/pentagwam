use crate::defs::{Idx, Sym};

pub union Cell {
    pub(crate) functor: Functor,
    pub(crate) tagged: TaggedCell,
}

impl From<TaggedCell> for Cell {
    fn from(tagged: TaggedCell) -> Self {
        Self { tagged }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TaggedCell {
    /// A reference (usually represents a variable).
    Ref(Idx),
    /// A record with an index to its functor.
    Rcd(Idx),
}

impl Cell {
    pub fn r#ref(idx: impl Into<Idx>) -> Self {
        TaggedCell::Ref(idx.into()).into()
    }

    pub fn rcd(idx: impl Into<Idx>) -> Self {
        TaggedCell::Rcd(idx.into()).into()
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
