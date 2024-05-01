use crate::{cell::Cell, defs::CellRef, mem::Mem};

pub fn unify(mem: &mut Mem, t1_ref: CellRef, t2_ref: CellRef) -> bool {
    let (t1_ref, t1) = mem.resolve_ref_to_ref_and_cell(t1_ref);
    let (t2_ref, t2) = mem.resolve_ref_to_ref_and_cell(t2_ref);

    // Step 1: ensure cell types match.
    match (t1, t2) {
        (Cell::Sig(f1), Cell::Sig(f2)) => f1 == f2,
        (Cell::Int(i1), Cell::Int(i2)) => i1 == i2,
        (Cell::Sym(s1), Cell::Sym(s2)) => s1 == s2,
        // Two unbound variables:
        (Cell::Ref(ref1), Cell::Ref(ref2)) => {
            tracing::trace!(
                "unifying var `{}` and var `{}`",
                mem.human_readable_var_name(ref1),
                mem.human_readable_var_name(ref2),
            );
            // Make t1 point to t2 (arbitrary choice).
            mem.cell_write(t1_ref, Cell::Ref(t2_ref));
            // TODO: record variable binding in trail.
            true
        }
        (Cell::Ref(r), concrete) => {
            tracing::trace!(
                "unifying var `{}` and concrete `{}`",
                mem.human_readable_var_name(r),
                mem.display_cell(concrete),
            );
            // Make the var point to the concrete value.
            mem.cell_write(t1_ref, Cell::Ref(t2_ref));
            // TODO: record variable binding in trail.
            true
        }
        (concrete, Cell::Ref(r)) => {
            tracing::trace!(
                "unifying concrete `{}` and var `{}`",
                mem.display_cell(concrete),
                mem.human_readable_var_name(r),
            );
            // Make the var point to the concrete value.
            mem.cell_write(t2_ref, Cell::Ref(t1_ref));
            // TODO: record variable binding in trail.
            true
        }
        (Cell::Lst(car1_ref), Cell::Lst(car2_ref)) => {
            if car1_ref == t1_ref || car2_ref == t2_ref {
                // One is nil, so both must be nil.
                return car1_ref == t1_ref && car2_ref == t2_ref;
            }

            // Unify the head cells.
            if !unify(mem, car1_ref, car2_ref) {
                return false;
            }

            let cdr1_ref = car1_ref + 1;
            let cdr2_ref = car2_ref + 1;

            // Unify the tail cells.
            unify(mem, cdr1_ref, cdr2_ref)
        }
        (Cell::Nil, Cell::Nil) => true,
        (Cell::Rcd(f1_ref), Cell::Rcd(f2_ref)) => {
            // Step 2: ensure functors match.

            let Cell::Sig(f1) = mem.cell_read(f1_ref) else {
                tracing::warn!("expected functor cell at index {f1_ref}");
                return false;
            };

            let Cell::Sig(f2) = mem.cell_read(f2_ref) else {
                tracing::warn!("expected functor cell at index {f2_ref}");
                return false;
            };

            tracing::trace!(
                "unifying compound term {} and compound term {}",
                mem.display_cell(Cell::Sig(f1)),
                mem.display_cell(Cell::Sig(f2))
            );

            if f1 != f2 {
                // Different functors; cannot unify.
                return false;
            }

            // Add 1 to skip past the functor cell.
            let base1 = f1_ref + 1;
            let base2 = f2_ref + 1;

            // Step 3: unify arguments.
            for i in 0..f1.arity as usize {
                let arg1_ref = base1 + i;
                let arg2_ref = base2 + i;
                if !unify(mem, arg1_ref, arg2_ref) {
                    return false;
                }
            }

            // All arguments unified so `t1` and `t2` unify.
            true
        }
        // I'm writing these out explicitly so that when I add a new `Cell`
        // variant I'll get a compilation error and know to update this match
        // expression.
        (Cell::Rcd(_), Cell::Int(_)) => false,
        (Cell::Rcd(_), Cell::Sym(_)) => false,
        (Cell::Int(_), Cell::Rcd(_)) => false,
        (Cell::Int(_), Cell::Sym(_)) => false,
        (Cell::Sym(_), Cell::Rcd(_)) => false,
        (Cell::Sym(_), Cell::Int(_)) => false,
        (Cell::Rcd(_), Cell::Sig(_)) => false,
        (Cell::Int(_), Cell::Sig(_)) => false,
        (Cell::Sym(_), Cell::Sig(_)) => false,
        (Cell::Sig(_), Cell::Rcd(_)) => false,
        (Cell::Sig(_), Cell::Int(_)) => false,
        (Cell::Sig(_), Cell::Sym(_)) => false,
        (Cell::Rcd(_), Cell::Lst(_)) => false,
        (Cell::Int(_), Cell::Lst(_)) => false,
        (Cell::Sym(_), Cell::Lst(_)) => false,
        (Cell::Sig(_), Cell::Lst(_)) => false,
        (Cell::Lst(_), Cell::Rcd(_)) => false,
        (Cell::Lst(_), Cell::Int(_)) => false,
        (Cell::Lst(_), Cell::Sym(_)) => false,
        (Cell::Lst(_), Cell::Sig(_)) => false,
        (Cell::Rcd(_), Cell::Nil) => false,
        (Cell::Int(_), Cell::Nil) => false,
        (Cell::Sym(_), Cell::Nil) => false,
        (Cell::Sig(_), Cell::Nil) => false,
        (Cell::Lst(_), Cell::Nil) => false,
        (Cell::Nil, Cell::Rcd(_)) => false,
        (Cell::Nil, Cell::Int(_)) => false,
        (Cell::Nil, Cell::Sym(_)) => false,
        (Cell::Nil, Cell::Sig(_)) => false,
        (Cell::Nil, Cell::Lst(_)) => false,
    }
}
