use core::{fmt, panic};
use std::{borrow::Cow, collections::BTreeMap};

use tracing::instrument;

use crate::{
    cell::{Cell, Functor},
    defs::{Idx, Sym},
};

pub struct Mem {
    pub(crate) heap: Vec<Cell>,
    /// Interned symbols.
    pub(crate) symbols: Vec<String>,
    /// Maps variable names to their index in the heap.
    pub(crate) var_indices: BTreeMap<Sym, Idx>,
}

impl Mem {
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            symbols: Vec::new(),
            var_indices: BTreeMap::new(),
        }
    }

    /// Create a value which can be displayed representing the term stored at
    /// index `idx`.
    pub fn display_term(&self, idx: Idx) -> DisplayTerm {
        DisplayTerm { idx, mem: self }
    }

    pub(crate) fn display_cell(&self, cell: Cell) -> DisplayCell {
        DisplayCell { cell, mem: self }
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

    pub fn push(&mut self, cell: Cell) -> Idx {
        let idx = self.heap.len().into();
        tracing::trace!(
            "pushing cell `{}` into index {idx}",
            self.display_cell(cell)
        );
        self.heap.push(cell);
        idx
    }

    pub fn var_name_from_idx(&self, idx: Idx) -> Option<Sym> {
        self.var_indices
            .iter()
            .find_map(|(sym, i)| (*i == idx).then_some(*sym))
    }

    pub fn var_idx_from_sym(&self, name: Sym) -> Option<Idx> {
        self.var_indices.get(&name).copied()
    }

    pub fn human_readable_var_name(&self, idx: Idx) -> Cow<str> {
        if let Some(sym) = self.var_name_from_idx(idx) {
            sym.resolve(self).into()
        } else {
            format!("_{}", idx.usize()).into()
        }
    }

    /// If the name is already associated with a variable, return the index of
    /// that variable. Otherwise, intern the name and push a variable with that
    /// name onto the heap. Return it's index.
    #[instrument(level = "trace", skip(self), ret)]
    pub fn push_var(&mut self, name: &str) -> Idx {
        let sym = self.intern_sym(name);
        // This is either a new index or the index of the existing variable.
        let var_idx = *self
            .var_indices
            .entry(sym)
            .or_insert_with(|| self.heap.len().into());
        self.heap.push(Cell::Ref(var_idx));
        self.var_indices.insert(sym, var_idx);
        var_idx
    }

    /// Create a fresh variable and return its index. Do not associate a name
    /// with the variable.
    #[instrument(level = "trace", skip(self), ret)]
    pub fn push_fresh_var(&mut self) -> Idx {
        let idx = self.heap.len().into();
        self.heap.push(Cell::Ref(idx));
        idx
    }

    pub fn cell_read(&self, idx: impl Into<Idx>) -> Cell {
        self.heap[idx.into().usize()]
    }

    pub fn cell_write(&mut self, idx: Idx, cell: Cell) {
        tracing::trace!("HEAP[{idx}] <- {}", self.display_cell(cell));
        self.heap[idx.usize()] = cell;
    }

    /// Follow references until a concrete value is found.
    #[instrument(level = "trace", skip(self), ret)]
    pub fn follow_refs(&self, cell_idx: Idx) -> Cell {
        let (_idx, tagged) = self.follow_refs_with_idx(cell_idx);
        tagged
    }

    /// Follow references until a concrete value is found. Returns the index of
    /// the concrete value and the concrete value itself.
    #[instrument(level = "trace", skip(self), ret)]
    pub fn follow_refs_with_idx(&self, mut cell_idx: Idx) -> (Idx, Cell) {
        loop {
            match self.heap[cell_idx.usize()] {
                this @ Cell::Ref(next) if next == cell_idx => {
                    // Unbound variable, just return
                    return (cell_idx, this);
                }
                Cell::Ref(next) => {
                    // It's a reference that points somewhere new, follow it.
                    cell_idx = next;
                }
                // If it's not a reference, it's a concrete value.
                other => return (cell_idx, other),
            }
        }
    }
}

impl Default for Mem {
    fn default() -> Self {
        Self::new()
    }
}

impl Sym {
    pub fn resolve<'a>(&self, mem: &'a Mem) -> &'a str {
        &mem.symbols[self.usize()]
    }
}

pub struct DisplayTerm<'a> {
    idx: Idx,
    mem: &'a Mem,
}

impl std::fmt::Display for DisplayTerm<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mem.cell_read(self.idx) {
            Cell::Int(i) => write!(f, "{i}"),
            Cell::Sym(sym) => write!(f, "{}", sym.resolve(self.mem)),
            Cell::Sig(functor) => {
                write!(f, "<{}/{}>", functor.sym.resolve(self.mem), functor.arity)
            }
            Cell::Ref(r) if r == self.idx => {
                if let Some(sym) = self.mem.var_name_from_idx(self.idx) {
                    write!(f, "{}", sym.resolve(self.mem))
                } else {
                    write!(f, "_{}", self.idx.usize())
                }
            }
            Cell::Ref(r) => write!(f, "{}", self.mem.display_term(r)),
            Cell::Rcd(start) => {
                let Cell::Sig(Functor { sym, arity }) = self.mem.cell_read(start) else {
                    panic!(
                        "expected a functor at index {start} but found {:?}",
                        self.mem.cell_read(start)
                    );
                };
                let functor_name = sym.resolve(self.mem);
                write!(f, "{functor_name}(")?;
                for arg_idx in 0..arity as usize {
                    if arg_idx != 0 {
                        write!(f, ", ")?;
                    }
                    let base = start.usize() + 1; // Skip functor.
                    let arg_idx = Idx::new(base + arg_idx);
                    write!(f, "{}", self.mem.display_term(arg_idx))?;
                }
                write!(f, ")")?;
                Ok(())
            }
        }
    }
}

pub(crate) struct DisplayCell<'a> {
    cell: Cell,
    mem: &'a Mem,
}

impl std::fmt::Display for DisplayCell<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.cell {
            Cell::Sym(sym) => write!(f, "Sym({})", sym.resolve(self.mem)),
            Cell::Sig(functor) => {
                write!(
                    f,
                    "Sig({}/{})",
                    functor.sym.resolve(self.mem),
                    functor.arity
                )
            }
            Cell::Int(..) | Cell::Ref(..) | Cell::Rcd(..) => write!(f, "{}", self.cell),
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
        Cell::Rcd(1.into()), // 0
        Cell::Sig(h2),       // 1
        Cell::Ref(2.into()), // 2
        Cell::Ref(3.into()), // 3
        Cell::Rcd(5.into()), // 4
        Cell::Sig(f1),       // 5
        Cell::Ref(3.into()), // 6
        Cell::Rcd(8.into()), // 7
        Cell::Sig(p3),       // 8
        Cell::Ref(2.into()), // 9
        Cell::Rcd(1.into()), // 10
        Cell::Rcd(5.into()), // 11
    ];

    let s = mem.display_term(7.into());
    assert_eq!(s.to_string(), "p(_2, h(_2, _3), f(_3))");
}

#[test]
fn unify_two_values() {
    let mut mem = Mem::new();

    let f1 = mem.intern_functor("f", 1);

    mem.heap = vec![
        // Build term t1: `f(_1)`
        Cell::Rcd(1.into()), // 0
        Cell::Sig(f1),       // 1
        Cell::Ref(2.into()), // 2
        //  Build term t2: `f(123)`
        Cell::Rcd(4.into()), // 3
        Cell::Sig(f1),       // 4
        Cell::Int(123),      // 5
    ];

    let t1_idx = Idx::new(0);
    let t2_idx = Idx::new(3);

    let t1 = mem.follow_refs(t1_idx);
    let t2 = mem.follow_refs(t2_idx);

    // Step 1: ensure cell types match.
    match (t1, t2) {
        (Cell::Rcd(idx1), Cell::Rcd(idx2)) => {
            // Step 2: ensure functors match.

            let Cell::Sig(f1) = mem.cell_read(idx1) else {
                panic!(
                    "expected a functor at index {:?} but found {:?}",
                    idx1,
                    mem.cell_read(idx1)
                );
            };
            let Cell::Sig(f2) = mem.cell_read(idx2) else {
                panic!(
                    "expected a functor at index {:?} but found {:?}",
                    idx1,
                    mem.cell_read(idx1)
                );
            };

            assert_eq!(f1, f2);

            // Step 3: unify arguments.
            for i in 0..f1.arity {
                let arg1_idx = Idx::new(idx1.usize() + 1 + i as usize);
                let arg2_idx = Idx::new(idx2.usize() + 1 + i as usize);

                // Follow pointers if necessary.
                match (mem.follow_refs(arg1_idx), mem.follow_refs(arg2_idx)) {
                    (Cell::Ref(r1), Cell::Ref(_r2)) => {
                        // Make r1 point to r2 (arbitrary choice).
                        mem.cell_write(r1, mem.follow_refs(arg2_idx));
                        // TODO: record variable binding in trail.
                    }
                    (Cell::Int(i1), Cell::Int(i2)) => {
                        assert_eq!(i1, i2);
                    }
                    (Cell::Sym(s1), Cell::Sym(s2)) => {
                        assert_eq!(s1, s2);
                    }
                    (Cell::Ref(_), _concrete) => {
                        // Make the var point to the concrete value.
                        mem.cell_write(arg1_idx, Cell::Ref(arg2_idx));
                        // TODO: record variable binding in trail.
                    }
                    (_concrete, Cell::Ref(_)) => {
                        // Make the var point to the concrete value.
                        mem.cell_write(arg2_idx, Cell::Ref(arg1_idx));
                        // TODO: record variable binding in trail.

                        assert_eq!(mem.follow_refs(arg1_idx), Cell::Int(123));
                        assert_eq!(mem.follow_refs(arg2_idx), Cell::Int(123));
                    }
                    (Cell::Rcd(_), Cell::Rcd(_)) => {
                        todo!("recurse (out of scope of this example)");
                    }
                    _ => panic!("Unification fails: {t1:?} != {t2:?}"),
                }
            }
        }
        (Cell::Int(i1), Cell::Int(i2)) if i1 == i2 => {}
        (Cell::Sym(s1), Cell::Sym(s2)) if s1 == s2 => {}
        (Cell::Ref(_), _) | (_, Cell::Ref(_)) => {
            todo!("Unify variables (out of scope of this example)");
        }
        _ => panic!("Unification fails: {t1:?} != {t2:?}"),
    }
}
