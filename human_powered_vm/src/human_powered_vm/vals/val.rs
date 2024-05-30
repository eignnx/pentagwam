use std::fmt;

use super::*;

#[derive(Debug, From, Clone, Serialize, Deserialize)]
pub enum Val {
    #[from]
    CellRef(CellRef),
    Usize(usize),
    I32(i32),
    Cell(Cell),
}

impl Default for Val {
    fn default() -> Self {
        Self::Cell(Cell::Nil)
    }
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Val::CellRef(cell_ref) => write!(f, "{cell_ref}"),
            Val::Usize(u) => write!(f, "{u}"),
            Val::I32(i) => write!(f, "{i:+}"),
            Val::Cell(cell) => write!(f, "{cell:?}"),
        }
    }
}

impl Val {
    pub fn ty(&self) -> ValTy {
        match self {
            Val::CellRef(_) => ValTy::CellRef,
            Val::Usize(_) => ValTy::Usize,
            Val::I32(_) => ValTy::I32,
            Val::Cell(_) => ValTy::AnyCellVal,
        }
    }

    pub fn expect_cell_ref(&self) -> Result<CellRef> {
        match self {
            Val::CellRef(cell_ref) => Ok(*cell_ref),
            other => Err(Error::TypeError {
                expected: "CellRef".into(),
                received: other.ty(),
            }),
        }
    }

    pub fn expect_i32(&self) -> Result<i32> {
        match self {
            Val::I32(i) => Ok(*i),
            other => Err(Error::TypeError {
                expected: "i32".into(),
                received: other.ty(),
            }),
        }
    }

    pub fn expect_usize(&self) -> Result<usize> {
        match self {
            Val::Usize(u) => Ok(*u),
            other => Err(Error::TypeError {
                expected: "usize".into(),
                received: other.ty(),
            }),
        }
    }

    pub fn expect_cell(&self) -> Result<Cell> {
        match self {
            Val::Cell(cell) => Ok(*cell),
            other => Err(Error::TypeError {
                expected: "Cell".into(),
                received: other.ty(),
            }),
        }
    }
}
