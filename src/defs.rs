#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Idx(pub(crate) usize);

impl From<usize> for Idx {
    fn from(idx: usize) -> Self {
        Self(idx)
    }
}

impl From<Idx> for usize {
    fn from(idx: Idx) -> usize {
        idx.0
    }
}
