use core::fmt;
use std::collections::HashMap;

use crate::{
    cell::{Cell, Functor, TaggedCell},
    defs::{Idx, Sym},
};

pub struct Mem {
    pub(crate) heap: Vec<Cell>,
    pub(crate) symbols: Vec<String>,
    pub(crate) var_names: HashMap<Idx, Sym>,
}

impl Mem {
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            symbols: Vec::new(),
            var_names: HashMap::new(),
        }
    }

    /// Create a value which can be displayed representing the term stored at
    /// index `idx`.
    ///
    /// # Safety
    /// A **tagged** cell must exist at the given index. That is, the cell must
    /// contain a `Cell::tagged` variant.
    pub unsafe fn display_tagged_cell(&self, idx: Idx) -> DisplayTaggedCell {
        DisplayTaggedCell { idx, mem: self }
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

    pub fn push(&mut self, cell: impl Into<Cell>) -> Idx {
        let idx = self.heap.len().into();
        self.heap.push(cell.into());
        idx
    }

    /// Will intern a new variable name and return the index of the new variable
    /// or lookup an existing variable name and return the index of the existing
    /// variable.
    pub fn push_var(&mut self, name: &str) -> Idx {
        let x = self.intern_sym(name); ////////////////
        let idx = self.heap.len().into();
        self.var_names.insert(idx, ..);
        self.heap.push(TaggedCell::Ref(idx).into());
        idx
    }

    /// # Safety
    /// The caller must guaruntee that the cell at the given index is a tagged
    /// cell.
    pub unsafe fn tagged_cell(&self, idx: Idx) -> TaggedCell {
        // SAFETY: Guarunteed by function precondition.
        unsafe { self.heap[idx.usize()].tagged }
    }

    pub fn cell_write(&mut self, idx: Idx, cell: impl Into<Cell>) {
        self.heap[idx.usize()] = cell.into();
    }

    /// # Safety
    /// The caller must guaruntee that the cell at the given index is a
    /// `Cell.functor` variant.
    pub unsafe fn functor_cell(&self, idx: Idx) -> Functor {
        // SAFETY: Guarunteed by function precondition.
        unsafe { self.heap[idx.usize()].functor }
    }

    /// Follow references until a concrete value is found.
    pub fn follow_ref(&self, tagged_cell: TaggedCell) -> TaggedCell {
        let (_idx, tagged) = self.follow_ref_with_idx(tagged_cell);
        tagged
    }

    /// Follow references until a concrete value is found. Also returns the
    /// found cell's index.
    pub fn follow_ref_with_idx(&self, tagged_cell: TaggedCell) -> (Idx, TaggedCell) {
        match tagged_cell {
            TaggedCell::Ref(r) => {
                // SAFETY: A `TaggedCell::Ref` is guaranteed to point to a
                // tagged cell because the serialization format guarantees that
                // a `Ref` always points to a tagged cell.
                unsafe { self.follow_ref_idx(r) }
            }
            other => (Idx::new(0), other),
        }
    }

    /// Follow references until a concrete value is found. Returns the index of
    /// the concrete value and the concrete value itself.
    ///
    /// # Safety
    /// The caller must guaruntee that the cell at the given index is a tagged
    /// cell.
    pub unsafe fn follow_ref_idx(&self, mut tagged_cell_idx: Idx) -> (Idx, TaggedCell) {
        loop {
            // SAFETY: Assuming a valid serialization of the value being
            // represented, a `Ref` is guaranteed to point to a tagged cell.
            match unsafe { self.tagged_cell(tagged_cell_idx) } {
                this @ TaggedCell::Ref(next) if next == tagged_cell_idx => {
                    // Unbound variable, just return
                    return (tagged_cell_idx, this);
                }
                TaggedCell::Ref(next) => {
                    // It's a reference that points somewhere new, follow it.
                    tagged_cell_idx = next;
                }
                // If it's not a reference, it's a concrete value.
                other => return (tagged_cell_idx, other),
            }
        }
    }
}

impl Sym {
    pub fn resolve<'a>(&self, mem: &'a Mem) -> &'a str {
        &mem.symbols[self.idx as usize]
    }
}

pub struct DisplayTaggedCell<'a> {
    idx: Idx,
    mem: &'a Mem,
}

impl std::fmt::Display for DisplayTaggedCell<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: By constructing a `DisplayTaggedCell` with a valid index, the
        // cell at that index is guaranteed by the constructor's caller to be a
        // tagged cell.
        match unsafe { self.mem.heap[self.idx.usize()].tagged } {
            TaggedCell::Int(i) => write!(f, "{i}"),
            TaggedCell::Sym(sym) => write!(f, "{}", sym.resolve(self.mem)),
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

#[test]
fn unify_two_values() {
    let mut mem = Mem::new();

    let f1 = mem.intern_functor("f", 1);

    mem.heap = vec![
        // Build term t1: `f(_1)`
        Cell::rcd(1),      // 0
        Cell::functor(f1), // 1
        Cell::r#ref(2),    // 2
        // Build term t2: `f(123)`
        Cell::rcd(4),      // 3
        Cell::functor(f1), // 4
        Cell::int(123),    // 5
    ];

    let t1 = Idx::new(0);
    let t2 = Idx::new(3);

    // SAFETY: The cell at index `t1` IS a tagged cell.
    let t1 = unsafe { mem.tagged_cell(t1) };
    // SAFETY: The cell at index `t2` IS a tagged cell.
    let t2 = unsafe { mem.tagged_cell(t2) };

    let (t1_idx, t1) = mem.follow_ref_with_idx(t1);
    let (t2_idx, t2) = mem.follow_ref_with_idx(t2);

    // Step 1: ensure cell types match.
    match (t1, t2) {
        (TaggedCell::Rcd(idx1), TaggedCell::Rcd(idx2)) => {
            // Step 2: ensure functors match.

            // SAFETY: Since `t1` was a tagged cell, the cell at `idx1` is a
            // functor.
            let f1 = unsafe { mem.functor_cell(idx1) };

            // SAFETY: Since `t2` was a tagged cell, the cell at `idx2` is a
            // functor.
            let f2 = unsafe { mem.functor_cell(idx2) };

            assert_eq!(f1, f2);

            // Step 3: unify arguments.
            for i in 0..f1.arity {
                let arg1_idx = Idx::new(idx1.usize() + 1 + i as usize);
                let arg2_idx = Idx::new(idx2.usize() + 1 + i as usize);

                // SAFETY: Since `t1` was a tagged cell, the cell at `arg1` is a
                // tagged cell.
                let arg1 = unsafe { mem.tagged_cell(arg1_idx) };

                // SAFETY: Since `t2` was a tagged cell, the cell at `arg2` is a
                // tagged cell.
                let arg2 = unsafe { mem.tagged_cell(arg2_idx) };

                // Follow pointers if necessary.
                match (mem.follow_ref(arg1), mem.follow_ref(arg2)) {
                    (TaggedCell::Ref(r1), TaggedCell::Ref(_r2)) => {
                        // Make r1 point to r2 (arbitrary choice).
                        mem.cell_write(r1, mem.follow_ref(arg2));
                        // TODO: record variable binding in trail.
                    }
                    (TaggedCell::Int(i1), TaggedCell::Int(i2)) => {
                        assert_eq!(i1, i2);
                    }
                    (TaggedCell::Sym(s1), TaggedCell::Sym(s2)) => {
                        assert_eq!(s1, s2);
                    }
                    (TaggedCell::Ref(_), _concrete) => {
                        // Make the var point to the concrete value.
                        mem.cell_write(arg1_idx, Cell::r#ref(arg2_idx));
                        // TODO: record variable binding in trail.
                    }
                    (_concrete, TaggedCell::Ref(_)) => {
                        // Make the var point to the concrete value.
                        mem.cell_write(arg2_idx, Cell::r#ref(arg1_idx));
                        // TODO: record variable binding in trail.

                        // SAFETY: arg1_idx points to a tagged cell
                        // because it was created from the index of a tagged
                        // cell.
                        unsafe {
                            assert!(matches!(
                                mem.follow_ref_idx(arg1_idx).1,
                                TaggedCell::Int(123)
                            ));
                        }
                        // SAFETY: arg2_idx points to a tagged cell because it's
                        // cell was just overwritten with a reference to
                        // arg1_idx (and a reference is a tagged cell).
                        unsafe {
                            assert!(matches!(
                                mem.follow_ref_idx(arg2_idx).1,
                                TaggedCell::Int(123)
                            ));
                        }
                    }
                    (TaggedCell::Rcd(_), TaggedCell::Rcd(_)) => {
                        todo!("recurse (out of scope of this example)");
                    }
                    _ => panic!("Unification fails: {t1:?} != {t2:?}"),
                }
            }
        }
        (TaggedCell::Int(i1), TaggedCell::Int(i2)) if i1 == i2 => {}
        (TaggedCell::Sym(s1), TaggedCell::Sym(s2)) if s1 == s2 => {}
        (TaggedCell::Ref(_), _) | (_, TaggedCell::Ref(_)) => {
            todo!("Unify variables (out of scope of this example)");
        }
        _ => panic!("Unification fails: {t1:?} != {t2:?}"),
    }
}
