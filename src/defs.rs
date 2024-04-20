type Index = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Idx(Index);

impl From<usize> for Idx {
    fn from(idx: usize) -> Self {
        Self(idx.try_into().unwrap())
    }
}

impl From<i32> for Idx {
    fn from(idx: i32) -> Self {
        Self(idx.try_into().unwrap())
    }
}

impl From<Index> for Idx {
    fn from(idx: Index) -> Self {
        Self(idx)
    }
}

impl Idx {
    pub fn new(idx: usize) -> Self {
        idx.into()
    }

    pub fn usize(self) -> usize {
        self.0.try_into().unwrap()
    }
}

impl std::ops::Add<Idx> for Idx {
    type Output = Idx;

    fn add(self, rhs: Idx) -> Self::Output {
        (self.0 + rhs.0).into()
    }
}

impl std::ops::Add<usize> for Idx {
    type Output = Idx;

    fn add(self, rhs: usize) -> Self::Output {
        (self.0 + rhs as u32).into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Sym {
    pub(crate) idx: Index,
}

impl Sym {
    pub fn new(idx: usize) -> Self {
        Self {
            idx: idx.try_into().unwrap(),
        }
    }
}
