use core::{fmt, panic};
use std::{borrow::Cow, collections::BTreeMap};

use tracing::instrument;

use crate::{
    cell::{Cell, Functor},
    defs::{CellRef, Sym},
};

pub struct Mem {
    pub(crate) heap: Vec<Cell>,
    /// Interned symbols.
    pub(crate) symbols: Vec<String>,
    /// Maps variable names to their index in the heap.
    pub(crate) var_indices: BTreeMap<Sym, CellRef>,
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
    /// `cell_ref`
    pub fn display_term(&self, cell_ref: CellRef) -> DisplayTerm {
        DisplayTerm {
            cell_ref,
            mem: self,
        }
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

    pub fn push(&mut self, cell: Cell) -> CellRef {
        let cell_ref = self.heap.len().into();
        tracing::trace!(
            "pushing cell `{}` into cell {cell_ref}",
            self.display_cell(cell)
        );
        self.heap.push(cell);
        cell_ref
    }

    pub fn var_name_from_cell_ref(&self, cell_ref: CellRef) -> Option<Sym> {
        self.var_indices
            .iter()
            .find_map(|(sym, r)| (*r == cell_ref).then_some(*sym))
    }

    pub fn var_ref_from_sym(&self, name: Sym) -> Option<CellRef> {
        self.var_indices.get(&name).copied()
    }

    pub fn var_ref_from_name(&self, name: &str) -> Option<CellRef> {
        self.var_ref_from_sym(Sym::new(self.symbols.iter().position(|s| s == name)?))
    }

    pub fn cell_from_var_name(&self, name: &str) -> Option<Cell> {
        self.var_ref_from_name(name)
            .map(|r| self.resolve_ref_to_cell(r))
    }

    pub fn human_readable_var_name(&self, cell_ref: CellRef) -> Cow<str> {
        if let Some(sym) = self.var_name_from_cell_ref(cell_ref) {
            sym.resolve(self).into()
        } else {
            format!("_{}", cell_ref.usize()).into()
        }
    }

    /// If the name is already associated with a variable, return the index of
    /// that variable. Otherwise, intern the name and push a variable with that
    /// name onto the heap. Return it's index.
    #[instrument(level = "trace", skip(self), ret)]
    pub fn push_var(&mut self, name: &str) -> CellRef {
        let sym = self.intern_sym(name);
        // This is either a new index or the index of the existing variable.
        let ref_to_var = *self
            .var_indices
            .entry(sym)
            .or_insert_with(|| self.heap.len().into());
        self.heap.push(Cell::Ref(ref_to_var));
        self.var_indices.insert(sym, ref_to_var);
        ref_to_var
    }

    /// Create a fresh variable and return its index. Do not associate a name
    /// with the variable.
    #[instrument(level = "trace", skip(self), ret)]
    pub fn push_fresh_var(&mut self) -> CellRef {
        let fresh_ref = self.heap.len().into();
        self.heap.push(Cell::Ref(fresh_ref));
        fresh_ref
    }

    pub fn cell_read(&self, cell_ref: impl Into<CellRef>) -> Cell {
        self.heap[cell_ref.into().usize()]
    }

    pub fn cell_write(&mut self, cell_ref: CellRef, cell: Cell) {
        tracing::trace!("HEAP[{cell_ref}] <- {}", self.display_cell(cell));
        self.heap[cell_ref.usize()] = cell;
    }

    /// Follow references until a concrete value is found.
    #[instrument(level = "trace", skip(self), ret)]
    pub fn resolve_ref_to_cell(&self, cell_ref: CellRef) -> Cell {
        let (_cell_ref, tagged) = self.resolve_ref_to_ref_and_cell(cell_ref);
        tagged
    }

    /// Follow references until a concrete value is found. Returns the index of
    /// the concrete value and the concrete value itself.
    #[instrument(level = "trace", skip(self), ret)]
    pub fn resolve_ref_to_ref_and_cell(&self, mut cell_ref: CellRef) -> (CellRef, Cell) {
        loop {
            match self.cell_read(cell_ref) {
                this @ Cell::Ref(next) if next == cell_ref => {
                    // Unbound variable, just return
                    return (cell_ref, this);
                }
                Cell::Ref(next) => {
                    // It's a reference that points somewhere new, follow it.
                    cell_ref = next;
                }
                // If it's not a reference, it's a concrete value.
                other => return (cell_ref, other),
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
    cell_ref: CellRef,
    mem: &'a Mem,
}

impl std::fmt::Display for DisplayTerm<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mem.cell_read(self.cell_ref) {
            Cell::Int(i) => write!(f, "{i}"),
            Cell::Sym(sym) => write!(f, "{}", sym.resolve(self.mem)),
            Cell::Sig(functor) => {
                write!(f, "<{}/{}>", functor.sym.resolve(self.mem), functor.arity)
            }
            Cell::Ref(r) if r == self.cell_ref => {
                if let Some(sym) = self.mem.var_name_from_cell_ref(self.cell_ref) {
                    write!(f, "{}", sym.resolve(self.mem))
                } else {
                    write!(f, "_{}", self.cell_ref.usize())
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
                for arg_ref in 0..arity as usize {
                    if arg_ref != 0 {
                        write!(f, ", ")?;
                    }
                    let base = start.usize() + 1; // Skip functor.
                    let arg_ref = CellRef::new(base + arg_ref);
                    write!(f, "{}", self.mem.display_term(arg_ref))?;
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

    let t1_ref = CellRef::new(0);
    let t2_ref = CellRef::new(3);

    let t1 = mem.resolve_ref_to_cell(t1_ref);
    let t2 = mem.resolve_ref_to_cell(t2_ref);

    // Step 1: ensure cell types match.
    match (t1, t2) {
        (Cell::Rcd(ref1), Cell::Rcd(ref2)) => {
            // Step 2: ensure functors match.

            let Cell::Sig(f1) = mem.cell_read(ref1) else {
                panic!(
                    "expected a functor at index {:?} but found {:?}",
                    ref1,
                    mem.cell_read(ref1)
                );
            };
            let Cell::Sig(f2) = mem.cell_read(ref2) else {
                panic!(
                    "expected a functor at index {:?} but found {:?}",
                    ref1,
                    mem.cell_read(ref1)
                );
            };

            assert_eq!(f1, f2);

            // Step 3: unify arguments.
            for i in 0..f1.arity {
                let arg1_ref = CellRef::new(ref1.usize() + 1 + i as usize);
                let arg2_ref = CellRef::new(ref2.usize() + 1 + i as usize);

                // Follow pointers if necessary.
                match (
                    mem.resolve_ref_to_cell(arg1_ref),
                    mem.resolve_ref_to_cell(arg2_ref),
                ) {
                    (Cell::Ref(r1), Cell::Ref(_r2)) => {
                        // Make r1 point to r2 (arbitrary choice).
                        mem.cell_write(r1, mem.resolve_ref_to_cell(arg2_ref));
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
                        mem.cell_write(arg1_ref, Cell::Ref(arg2_ref));
                        // TODO: record variable binding in trail.
                    }
                    (_concrete, Cell::Ref(_)) => {
                        // Make the var point to the concrete value.
                        mem.cell_write(arg2_ref, Cell::Ref(arg1_ref));
                        // TODO: record variable binding in trail.

                        assert_eq!(mem.resolve_ref_to_cell(arg1_ref), Cell::Int(123));
                        assert_eq!(mem.resolve_ref_to_cell(arg2_ref), Cell::Int(123));
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
