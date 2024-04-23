use std::task::Poll;

use crate::{cell::Cell, defs::CellRef, mem::Mem};

pub struct Vm {
    mem: Mem,
    /// The worklist of items left to unify.
    stack: Vec<(CellRef, CellRef)>,
}

impl Vm {
    pub fn new(mem: Mem) -> Self {
        Self {
            mem,
            stack: Vec::new(),
        }
    }

    pub fn setup_unification(&mut self, t1_ref: CellRef, t2_ref: CellRef) {
        self.stack.clear();
        self.stack.push((t1_ref, t2_ref));
    }

    pub fn run_unification(&mut self) -> bool {
        loop {
            if let Poll::Ready(success) = self.unification_step() {
                return success;
            }
        }
    }

    pub fn unification_step(&mut self) -> Poll<bool> {
        let Some((t1_ref, t2_ref)) = self.stack.pop() else {
            return Poll::Ready(true);
        };

        let (t1_ref, t1) = self.mem.resolve_ref_to_ref_and_cell(t1_ref);
        let (t2_ref, t2) = self.mem.resolve_ref_to_ref_and_cell(t2_ref);

        match (t1, t2) {
            (Cell::Int(i1), Cell::Int(i2)) => Poll::Ready(i1 == i2 && self.stack.is_empty()),
            (Cell::Sym(s1), Cell::Sym(s2)) => Poll::Ready(s1 == s2 && self.stack.is_empty()),
            // If these were returned from the `resolve_ref_to_ref_and_cell` method
            // then t1 and t2 are unbound variables.
            (Cell::Ref(r1), Cell::Ref(r2)) => {
                // Make t1 point to t2 (arbitrary choice).
                self.mem.cell_write(t1_ref, Cell::Ref(t2_ref));
                Poll::Ready(self.stack.is_empty())
            }
            (Cell::Ref(r), concrete) => {
                // Make the var point to the concrete value.
                self.mem.cell_write(t1_ref, Cell::Ref(t2_ref));
                Poll::Ready(self.stack.is_empty())
            }
            (concrete, Cell::Ref(r)) => {
                // Make the var point to the concrete value.
                self.mem.cell_write(t2_ref, Cell::Ref(t1_ref));
                Poll::Ready(self.stack.is_empty())
            }
            (Cell::Rcd(f1_ref), Cell::Rcd(f2_ref)) => {
                let Cell::Sig(f1) = self.mem.cell_read(f1_ref) else {
                    tracing::warn!("expected functor cell at index {f1_ref}");
                    return Poll::Ready(false);
                };

                let Cell::Sig(f2) = self.mem.cell_read(f2_ref) else {
                    tracing::warn!("expected functor cell at index {f2_ref}");
                    return Poll::Ready(false);
                };

                if f1 != f2 {
                    // Different functors; cannot unify.
                    return Poll::Ready(false);
                }

                // Add 1 to skip past the functor cell.
                let base1 = f1_ref + 1;
                let base2 = f2_ref + 1;

                for i in 0..f1.arity as usize {
                    let arg1_ref = base1 + i;
                    let arg2_ref = base2 + i;
                    self.stack.push((arg1_ref, arg2_ref));
                }

                Poll::Pending
            }
            (Cell::Rcd(_), Cell::Int(_))
            | (Cell::Rcd(_), Cell::Sym(_))
            | (Cell::Rcd(_), Cell::Sig(_))
            | (Cell::Int(_), Cell::Rcd(_))
            | (Cell::Int(_), Cell::Sym(_))
            | (Cell::Int(_), Cell::Sig(_))
            | (Cell::Sym(_), Cell::Rcd(_))
            | (Cell::Sym(_), Cell::Int(_))
            | (Cell::Sym(_), Cell::Sig(_))
            | (Cell::Sig(_), Cell::Rcd(_))
            | (Cell::Sig(_), Cell::Int(_))
            | (Cell::Sig(_), Cell::Sym(_))
            | (Cell::Sig(_), Cell::Sig(_)) => Poll::Ready(false),
        }
    }
}
