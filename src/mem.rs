use crate::{
    cell::{Cell, CellVal, Functor},
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
                let Functor { sym, arity } = unsafe { self.heap[start.0].functor };
                let functor_name = sym.resolve(self);
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

    pub fn intern_sym(&mut self, text: impl AsRef<str>) -> Sym {
        if let Some(idx) = self.symbols.iter().position(|s| s == text.as_ref()) {
            Sym { idx }
        } else {
            let sym = Sym {
                idx: self.symbols.len(),
            };
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
        &mem.symbols[self.idx]
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

    let mut s = Vec::<u8>::new();
    unsafe { mem.write_tagged_cell(7.into(), &mut s).unwrap() };
    assert_eq!(String::from_utf8(s).unwrap(), "p(_2, h(_2, _3), f(_3))");
}
