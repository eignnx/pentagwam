use core::fmt;

use serde::{Deserialize, Serialize};

use crate::mem::{DisplayViaMem, Mem};

type UInt = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CellRef(UInt);

impl Default for CellRef {
    fn default() -> Self {
        Self(UInt::MAX)
    }
}

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

impl std::ops::AddAssign<usize> for CellRef {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs as u32;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
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

impl DisplayViaMem for Sym {
    fn display_via_mem(&self, f: &mut fmt::Formatter<'_>, mem: &Mem) -> fmt::Result {
        write!(f, "{}", self.resolve(mem))
    }
}
