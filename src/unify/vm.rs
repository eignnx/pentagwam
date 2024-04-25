use std::ops::ControlFlow;

use crate::{cell::Cell, defs::CellRef, mem::Mem};

pub struct Vm {
    pub mem: Mem,
    /// The worklist of items left to unify.
    worklist: Vec<Work>,
}

#[derive(Debug, Clone)]
struct Work {
    t1_ref: CellRef,
    t2_ref: CellRef,
    argc_remaining: usize,
}

impl Vm {
    pub fn new(mem: Mem) -> Self {
        Self {
            mem,
            worklist: Vec::new(),
        }
    }

    pub fn setup_unification(&mut self, t1_ref: CellRef, t2_ref: CellRef) {
        self.worklist.clear();
        self.worklist.push(Work {
            t1_ref,
            t2_ref,
            argc_remaining: 1,
        });
    }

    pub fn run_unification(&mut self) -> bool {
        loop {
            if let ControlFlow::Break(successfulness) = self.unification_step() {
                return successfulness;
            }
        }
    }

    pub fn unification_step(&mut self) -> ControlFlow<bool> {
        match self.worklist.last_mut() {
            None => ControlFlow::Break(true),
            Some(Work {
                argc_remaining: 0, ..
            }) => {
                // Finished unifying all the args of this compound term.
                tracing::trace!("popping");
                self.worklist.pop();
                ControlFlow::Continue(())
            }
            Some(Work {
                t1_ref,
                t2_ref,
                argc_remaining,
            }) => {
                tracing::trace!("argc_remaining={argc_remaining}");
                let t1_ref_cpy = *t1_ref;
                let t2_ref_cpy = *t2_ref;
                *argc_remaining -= 1;
                *t1_ref += 1;
                *t2_ref += 1;
                self.generic_unification_step(t1_ref_cpy, t2_ref_cpy)
            }
        }
    }

    fn generic_unification_step(&mut self, t1_ref: CellRef, t2_ref: CellRef) -> ControlFlow<bool> {
        tracing::trace!("unifying {t1_ref} and {t2_ref}");
        let (t1_ref, t1) = self.mem.resolve_ref_to_ref_and_cell(t1_ref);
        let (t2_ref, t2) = self.mem.resolve_ref_to_ref_and_cell(t2_ref);

        tracing::trace!(
            "({} ~ {})",
            self.mem.display_cell(t1),
            self.mem.display_cell(t2),
        );

        match (t1, t2) {
            (Cell::Int(..), Cell::Int(..))
            | (Cell::Sym(..), Cell::Sym(..))
            | (Cell::Sig(..), Cell::Sig(..)) => {
                if t1 == t2 {
                    ControlFlow::Continue(())
                } else {
                    ControlFlow::Break(false)
                }
            }
            // If these were returned from the `resolve_ref_to_ref_and_cell` method
            // then t1 and t2 are unbound variables.
            (Cell::Ref(..), Cell::Ref(..)) => {
                // Make t1 point to t2 (arbitrary choice).
                self.mem.cell_write(t1_ref, Cell::Ref(t2_ref));
                ControlFlow::Continue(())
            }
            (Cell::Ref(..), _concrete) => {
                // Make the var point to the concrete value.
                self.mem.cell_write(t1_ref, Cell::Ref(t2_ref));
                ControlFlow::Continue(())
            }
            (_concrete, Cell::Ref(..)) => {
                // Make the var point to the concrete value.
                self.mem.cell_write(t2_ref, Cell::Ref(t1_ref));
                ControlFlow::Continue(())
            }
            (Cell::Rcd(f1_ref), Cell::Rcd(f2_ref)) => {
                let Cell::Sig(f1) = self.mem.cell_read(f1_ref) else {
                    tracing::warn!("expected functor cell at index {f1_ref}");
                    return ControlFlow::Break(false);
                };

                let Cell::Sig(f2) = self.mem.cell_read(f2_ref) else {
                    tracing::warn!("expected functor cell at index {f2_ref}");
                    return ControlFlow::Break(false);
                };

                if f1 != f2 {
                    // Different functors; cannot unify.
                    return ControlFlow::Break(false);
                }

                tracing::trace!("pushing ({}~{} + {})", f1_ref + 1, f2_ref + 1, f1.arity);
                self.worklist.push(Work {
                    // Add 1 to skip past the functor cell.
                    t1_ref: f1_ref + 1,
                    t2_ref: f2_ref + 1,
                    argc_remaining: f1.arity as usize,
                });

                ControlFlow::Continue(())
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
            | (Cell::Sig(_), Cell::Sym(_)) => ControlFlow::Break(false),
        }
    }
}
