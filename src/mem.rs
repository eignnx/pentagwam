use crate::{
    cell::{Cell, CellVal, Functor},
    defs::Idx,
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

    pub unsafe fn write_tagged_cell(
        &self,
        idx: Idx,
        f: &mut impl std::io::Write,
    ) -> std::io::Result<()> {
        match unsafe { self.heap[idx.0].tagged } {
            CellVal::Ref(r) if r == idx => {
                write!(f, "_{}", idx.0)
            }
            CellVal::Ref(r) => unsafe { self.write_tagged_cell(r, f) },
            CellVal::Rcd(start) => {
                let Functor { id, arity } = unsafe { self.heap[start.0].functor };
                let functor_name = &self.symbols[id];
                write!(f, "{functor_name}(")?;
                for arg_idx in 0..arity as usize {
                    if arg_idx != 0 {
                        write!(f, ", ")?;
                    }
                    let base = start.0 + 1; // Skip functor.
                    let arg_idx = Idx::from(base + arg_idx);
                    self.write_tagged_cell(arg_idx, f)?;
                }
                write!(f, ")")?;
                Ok(())
            }
        }
    }

    pub fn intern_sym(&mut self, sym: &str) -> usize {
        if let Some(idx) = self.symbols.iter().position(|s| s == sym) {
            idx
        } else {
            let idx = self.symbols.len();
            self.symbols.push(sym.to_string());
            idx
        }
    }
}

#[test]
fn test_heap() {
    let mut heap = Mem::new();

    let h2: Functor = Functor {
        id: heap.intern_sym("h"),
        arity: 2,
    };
    let f1: Functor = Functor {
        id: heap.intern_sym("f"),
        arity: 1,
    };
    let p3: Functor = Functor {
        id: heap.intern_sym("p"),
        arity: 3,
    };

    heap.heap = vec![
        CellVal::Rcd(1.into()).into(), // 0
        Cell { functor: h2 },          // 1
        CellVal::Ref(2.into()).into(), // 2
        CellVal::Ref(3.into()).into(), // 3
        CellVal::Rcd(5.into()).into(), // 4
        Cell { functor: f1 },          // 5
        CellVal::Ref(3.into()).into(), // 6
        CellVal::Rcd(8.into()).into(), // 7
        Cell { functor: p3 },          // 8
        CellVal::Ref(2.into()).into(), // 9
        CellVal::Rcd(1.into()).into(), // 10
        CellVal::Rcd(5.into()).into(), // 11
    ];

    let mut s = Vec::<u8>::new();
    unsafe { heap.write_tagged_cell(7.into(), &mut s).unwrap() };
    assert_eq!(String::from_utf8(s).unwrap(), "p(_2, h(_2, _3), f(_3))");
}
