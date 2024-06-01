use core::{fmt, panic};
use std::{
    borrow::Cow,
    cell::{Ref, RefCell},
    collections::BTreeMap,
};

use tracing::instrument;

use crate::{
    cell::{Cell, Functor},
    defs::{CellRef, Sym},
};

pub struct Mem {
    pub(crate) heap: Vec<Cell>,
    /// Interned symbols.
    pub(crate) symbols: RefCell<Vec<String>>,
    /// Maps variable names to their index in the heap.
    pub(crate) var_indices: BTreeMap<Sym, CellRef>,
}

impl Mem {
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            symbols: RefCell::new(Vec::new()),
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

    #[track_caller]
    pub fn intern_sym(&self, text: impl AsRef<str>) -> Sym {
        let opt_pos = self
            .symbols
            .borrow()
            .iter()
            .position(|s| s == text.as_ref());
        if let Some(idx) = opt_pos {
            Sym::new(idx)
        } else {
            let sym = Sym::new(self.symbols.borrow().len());
            self.symbols.borrow_mut().push(text.as_ref().to_string());
            sym
        }
    }

    #[track_caller]
    pub fn intern_functor(&self, name: impl AsRef<str>, arity: u8) -> Functor {
        Functor {
            sym: self.intern_sym(name),
            arity,
        }
    }

    pub fn push(&mut self, cell: Cell) -> CellRef {
        let cell_ref = self.heap.len().into();
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
        let idx = self.symbols.borrow().iter().position(|s| s == name)?;
        self.var_ref_from_sym(Sym::new(idx))
    }

    pub fn cell_from_var_name(&self, name: &str) -> Option<Cell> {
        let cell_ref = self.var_ref_from_name(name)?;
        Some(self.resolve_ref_to_cell(cell_ref))
    }

    pub fn human_readable_var_name(&self, cell_ref: CellRef) -> Cow<str> {
        if let Some(sym) = self.var_name_from_cell_ref(cell_ref) {
            sym.resolve(self).to_owned().into()
        } else {
            format!("_{}", cell_ref.usize()).into()
        }
    }

    pub fn assign_name_to_var(&mut self, cell_ref: CellRef, name: &str) {
        let sym = self.intern_sym(name);
        self.var_indices.insert(sym, cell_ref);
    }

    /// If the name is already associated with a variable, return the index of
    /// that variable. Otherwise, intern the name and push a variable with that
    /// name onto the heap. Return it's index.
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

    #[track_caller]
    pub fn cell_read(&self, cell_ref: impl Into<CellRef>) -> Cell {
        self.heap[cell_ref.into().usize()]
    }

    pub fn try_cell_read(&self, cell_ref: impl Into<CellRef>) -> Option<Cell> {
        self.heap.get(cell_ref.into().usize()).copied()
    }

    #[instrument(level = "trace", skip(self))]
    pub fn cell_write(&mut self, cell_ref: CellRef, cell: Cell) {
        tracing::trace!("HEAP[{cell_ref}] <- {}", self.display_cell(cell));
        self.heap[cell_ref.usize()] = cell;
    }

    pub fn try_cell_write(&mut self, cell_ref: CellRef, cell: Cell) -> Option<()> {
        self.heap.get_mut(cell_ref.usize()).map(|slot| *slot = cell)
    }

    /// Follow references until a concrete value is found.
    #[instrument(level = "trace", skip(self), ret)]
    pub fn resolve_ref_to_cell(&self, cell_ref: CellRef) -> Cell {
        let (_cell_ref, tagged) = self.resolve_ref_to_ref_and_cell(cell_ref);
        tagged
    }

    /// Follow references until a concrete value is found. Returns the index of
    /// the concrete value and the concrete value itself.
    #[track_caller]
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

impl std::fmt::Debug for Mem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::collections::HashMap;
        let mut ref_tgts: HashMap<CellRef, Vec<CellRef>> = HashMap::new();
        for (i, cell) in self.heap.iter().enumerate() {
            match cell {
                Cell::Rcd(r) | Cell::Ref(r) => ref_tgts.entry(*r).or_default().push(i.into()),
                _ => {}
            }
        }

        for (i, cell) in self.heap.iter().enumerate() {
            let i = i.into();
            if self.var_indices.values().any(|&idx| idx == i) {
                let name = self.human_readable_var_name(i);
                write!(f, "`{name}`")?;
            }
            write!(f, "\t{i}\t{}", self.display_cell(*cell))?;
            if ref_tgts.contains_key(&i) {
                write!(f, " <-")?;
                for tgt in ref_tgts[&i].iter() {
                    write!(f, " {tgt}")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Sym {
    pub fn resolve<'a>(&self, mem: &'a Mem) -> Ref<'a, str> {
        Ref::map(mem.symbols.borrow(), |symbols| {
            symbols[self.usize()].as_ref()
        })
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
            Cell::Nil => write!(f, "[]"),
            Cell::Lst(mut r) => {
                write!(f, "[")?;
                let mut first = true;
                loop {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", self.mem.display_term(r))?;
                    match self.mem.cell_read(r + 1) {
                        Cell::Nil => break,
                        Cell::Lst(next) => {
                            r = next;
                        }
                        _ => {
                            write!(f, " | {}", self.mem.display_term(r))?;
                            break;
                        }
                    }
                    first = false;
                }
                write!(f, "]")?;
                Ok(())
            }
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
            Cell::Int(..) | Cell::Ref(..) | Cell::Rcd(..) | Cell::Lst(..) | Cell::Nil => {
                write!(f, "{}", self.cell)
            }
        }
    }
}

pub trait DisplayViaMem {
    fn display_via_mem(&self, f: &mut fmt::Formatter<'_>, mem: &Mem) -> fmt::Result;
}

impl<T: DisplayViaMem> DisplayViaMem for &T {
    fn display_via_mem(&self, f: &mut fmt::Formatter<'_>, mem: &Mem) -> fmt::Result {
        (*self).display_via_mem(f, mem)
    }
}

impl<T: DisplayViaMem> DisplayViaMem for Box<T> {
    fn display_via_mem(&self, f: &mut fmt::Formatter<'_>, mem: &Mem) -> fmt::Result {
        (**self).display_via_mem(f, mem)
    }
}

impl<T: DisplayViaMem> DisplayViaMem for std::rc::Rc<T> {
    fn display_via_mem(&self, f: &mut fmt::Formatter<'_>, mem: &Mem) -> fmt::Result {
        (**self).display_via_mem(f, mem)
    }
}

impl DisplayViaMem for String {
    fn display_via_mem(&self, f: &mut fmt::Formatter<'_>, _mem: &Mem) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub struct MemCtx<'m, T: DisplayViaMem> {
    mem: &'m Mem,
    value: &'m T,
}

impl Mem {
    pub fn display<'m, T: DisplayViaMem>(&'m self, value: &'m T) -> MemCtx<'m, T> {
        MemCtx { mem: self, value }
    }
}

impl<'m, T: DisplayViaMem> std::fmt::Display for MemCtx<'m, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.display_via_mem(f, self.mem)
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
