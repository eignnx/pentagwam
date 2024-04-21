type UInt = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellRef(UInt);

impl std::fmt::Display for CellRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}

impl From<usize> for CellRef {
    fn from(n: usize) -> Self {
        Self(n.try_into().unwrap())
    }
}

impl From<i32> for CellRef {
    fn from(n: i32) -> Self {
        Self(n.try_into().unwrap())
    }
}

impl From<UInt> for CellRef {
    fn from(n: UInt) -> Self {
        Self(n)
    }
}

impl CellRef {
    pub fn new(n: usize) -> Self {
        n.into()
    }

    pub fn usize(self) -> usize {
        self.0.try_into().unwrap()
    }
}

impl std::ops::Add<CellRef> for CellRef {
    type Output = CellRef;

    fn add(self, rhs: CellRef) -> Self::Output {
        (self.0 + rhs.0).into()
    }
}

impl std::ops::Add<usize> for CellRef {
    type Output = CellRef;

    fn add(self, rhs: usize) -> Self::Output {
        (self.0 + rhs as u32).into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sym(UInt);

impl Sym {
    pub fn new(n: usize) -> Self {
        Self(n.try_into().unwrap())
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
