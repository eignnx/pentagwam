type Index = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Idx(Index);

impl std::fmt::Display for Idx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}

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
pub struct Sym(Index);

impl Sym {
    pub fn new(idx: usize) -> Self {
        Self(idx.try_into().unwrap())
    }

    pub fn usize(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for Sym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}
