use crate::defs::{Idx, Sym};

#[repr(C)]
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
    /// An integer.
    Int(i32),
    /// A symbol.
    Sym(Sym),
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

    pub fn int(int: i32) -> Self {
        TaggedCell::Int(int).into()
    }

    pub fn sym(sym: Sym) -> Self {
        TaggedCell::Sym(sym).into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Functor {
    pub sym: Sym,
    pub arity: u8,
}
