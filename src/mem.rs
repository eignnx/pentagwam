use core::fmt;

use crate::{
    cell::{Cell, Functor, TaggedCell},
    defs::{Idx, Sym},
};

pub struct Mem {
    pub(crate) heap: Vec<Cell>,
    pub(crate) symbols: Vec<String>,
}

impl Mem {
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            symbols: Vec::new(),
        }
    }

    /// Create a value which can be displayed representing the term stored at
    /// index `idx`.
    ///
    /// # Safety
    /// A **tagged** cell must exist at the given index. That is, the cell must
    /// contain a `Cell::tagged` variant.
    pub unsafe fn display_tagged_cell(&self, idx: Idx) -> CellValDisplay {
        CellValDisplay { idx, mem: self }
    }

    pub fn intern_sym(&mut self, text: impl AsRef<str>) -> Sym {
        if let Some(idx) = self.symbols.iter().position(|s| s == text.as_ref()) {
            Sym::new(idx)
        } else {
            let sym = Sym::new(self.symbols.len());
            self.symbols.push(text.as_ref().to_string());
            sym
        }
    }

    pub fn intern_functor(&mut self, name: impl AsRef<str>, arity: u8) -> Functor {
        Functor {
            sym: self.intern_sym(name),
            arity,
        }
    }
}

impl Sym {
    pub fn resolve<'a>(&self, mem: &'a Mem) -> &'a str {
        &mem.symbols[self.idx as usize]
    }
}

pub struct CellValDisplay<'a> {
    idx: Idx,
    mem: &'a Mem,
}

impl std::fmt::Display for CellValDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: By constructing a `CellValDisplay` with a valid index, the
        // cell at that index is guaranteed by the constructor's caller to be a
        // tagged cell.
        match unsafe { self.mem.heap[self.idx.usize()].tagged } {
            TaggedCell::Ref(r) if r == self.idx => {
                write!(f, "_{}", self.idx.usize())
            }
            TaggedCell::Ref(r) => {
                // SAFETY: Assuming a valid serialization of the value being
                // represented, a `Ref` is guaranteed to point to a tagged cell.
                write!(f, "{}", unsafe { self.mem.display_tagged_cell(r) })
            }
            TaggedCell::Rcd(start) => {
                // SAFETY:
                // The cell at `start` contains a `Cell::functor` variant by
                // definition of the serialization format.
                let Functor { sym, arity } = unsafe { self.mem.heap[start.usize()].functor };
                let functor_name = sym.resolve(self.mem);
                write!(f, "{functor_name}(")?;
                for arg_idx in 0..arity as usize {
                    if arg_idx != 0 {
                        write!(f, ", ")?;
                    }
                    let base = start.usize() + 1; // Skip functor.
                    let arg_idx = Idx::new(base + arg_idx);
                    // SAFETY: Assuming a valid serialization of the value
                    // being represented, the next `arity` cells are
                    // guaranteed to be tagged cells.
                    write!(f, "{}", unsafe { self.mem.display_tagged_cell(arg_idx) })?;
                }
                write!(f, ")")?;
                Ok(())
            }
        }
    }
}

#[test]
fn test_heap() {
    let mut mem = Mem::new();

    let h2 = mem.intern_functor("h", 2);
    let f1 = mem.intern_functor("f", 1);
    let p3 = mem.intern_functor("p", 3);

    mem.heap = vec![
        Cell::rcd(1),      // 0
        Cell::functor(h2), // 1
        Cell::r#ref(2),    // 2
        Cell::r#ref(3),    // 3
        Cell::rcd(5),      // 4
        Cell::functor(f1), // 5
        Cell::r#ref(3),    // 6
        Cell::rcd(8),      // 7
        Cell::functor(p3), // 8
        Cell::r#ref(2),    // 9
        Cell::rcd(1),      // 10
        Cell::rcd(5),      // 11
    ];

    // SAFETY: The cell at index 7 is a tagged cell.
    let s = unsafe { mem.display_tagged_cell(7.into()) };
    assert_eq!(s.to_string(), "p(_2, h(_2, _3), f(_3))");
}
