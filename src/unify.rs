use crate::{cell::TaggedCell, mem::Mem};

pub fn unify(mem: &mut Mem, t1: TaggedCell, t2: TaggedCell) -> bool {
    let (t1_idx, t1) = mem.follow_ref_with_idx(t1);
    let (t2_idx, t2) = mem.follow_ref_with_idx(t2);

    // Step 1: ensure cell types match.
    match (t1, t2) {
        (TaggedCell::Int(i1), TaggedCell::Int(i2)) => i1 == i2,
        (TaggedCell::Sym(s1), TaggedCell::Sym(s2)) => s1 == s2,
        // Two unbound variables:
        (TaggedCell::Ref(_), TaggedCell::Ref(_)) => {
            // Make t1 point to t2 (arbitrary choice).
            mem.cell_write(t1_idx, TaggedCell::Ref(t2_idx));
            // TODO: record variable binding in trail.
            true
        }
        (TaggedCell::Ref(_), _concrete) => {
            // Make the var point to the concrete value.
            mem.cell_write(t1_idx, TaggedCell::Ref(t2_idx));
            // TODO: record variable binding in trail.
            true
        }
        (_concrete, TaggedCell::Ref(_)) => {
            // Make the var point to the concrete value.
            mem.cell_write(t2_idx, TaggedCell::Ref(t1_idx));
            // TODO: record variable binding in trail.
            true
        }
        (TaggedCell::Rcd(f_idx1), TaggedCell::Rcd(f_idx2)) => {
            // Step 2: ensure functors match.

            // SAFETY: Since `t1` was a tagged cell, the cell at `f_idx1` is a
            // functor by serialization format definition.
            let f1 = unsafe { mem.functor_cell(f_idx1) };

            // SAFETY: Since `t2` was a tagged cell, the cell at `f_idx2` is a
            // functor by serialization format definition.
            let f2 = unsafe { mem.functor_cell(f_idx2) };

            if f1 != f2 {
                // Different functors; cannot unify.
                return false;
            }

            // Add 1 to skip past the functor cell.
            let base1 = f_idx1 + 1;
            let base2 = f_idx2 + 1;

            // Step 3: unify arguments.
            for i in 0..f1.arity as usize {
                let arg1_idx = base1 + i;
                let arg2_idx = base2 + i;

                // SAFETY: Since `t1` was a tagged cell, the cell at `arg1_idx`
                // is a tagged cell due to the serialization format definition.
                let arg1 = unsafe { mem.tagged_cell(arg1_idx) };

                // SAFETY: Since `t2` was a tagged cell, the cell at `arg2_idx`
                // is a tagged cell due to the serialization format definition.
                let arg2 = unsafe { mem.tagged_cell(arg2_idx) };

                if !unify(mem, arg1, arg2) {
                    return false;
                }
            }

            // All arguments unified so `t1` and `t2` unify.
            true
        }
        // I'm writing these out explicitly so that when I add a new
        // `TaggedCell` variant I'll get a compilation error and know to update
        // this match expression.
        (TaggedCell::Rcd(_), TaggedCell::Int(_)) => false,
        (TaggedCell::Rcd(_), TaggedCell::Sym(_)) => false,
        (TaggedCell::Int(_), TaggedCell::Rcd(_)) => false,
        (TaggedCell::Int(_), TaggedCell::Sym(_)) => false,
        (TaggedCell::Sym(_), TaggedCell::Rcd(_)) => false,
        (TaggedCell::Sym(_), TaggedCell::Int(_)) => false,
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, ops::DerefMut, rc::Rc};

    use super::*;
    use crate::{defs::Sym, parse::parser};
    use chumsky::Parser;

    #[test]
    fn unify_ints() {
        let mut mem = Mem::new();
        let t1 = TaggedCell::Int(42);
        let t2 = TaggedCell::Int(42);
        assert!(unify(&mut mem, t1, t2));
    }

    #[test]
    fn unify_syms() {
        let mut mem = Mem::new();
        let t1 = TaggedCell::Sym(Sym::new(999));
        let t2 = TaggedCell::Sym(Sym::new(999));
        assert!(unify(&mut mem, t1, t2));

        let t3 = TaggedCell::Sym(Sym::new(1000));
        assert!(!unify(&mut mem, t1, t3));
    }

    #[track_caller]
    fn parse_and_unify(t1_src: &str, t2_src: &str) -> bool {
        let mem = Rc::new(RefCell::new(Mem::new()));
        let t1 = parser(mem.clone()).parse(t1_src).unwrap();
        let t2 = parser(mem.clone()).parse(t2_src).unwrap();
        let mut mem = mem.borrow_mut();
        unify(&mut mem, t1, t2)
    }

    #[test]
    fn unify_identical_compound_terms() {
        let t1_src = "person(alice, 29)";
        let t2_src = "person(alice, 29)";
        assert!(parse_and_unify(t1_src, t2_src));
    }

    #[test]
    fn unify_different_compound_terms() {
        let t1_src = "person(alice, 29)";
        let t2_src = "person(bob, 94)";
        assert!(!parse_and_unify(t1_src, t2_src));
    }

    #[test]
    fn unify_compound_terms_with_different_functors() {
        let t1_src = "person(alice, 29)";
        let t2_src = "inventory_item(adze, tool, weight(2, kg))";
        assert!(!parse_and_unify(t1_src, t2_src));
    }

    #[test]
    fn unify_compound_terms_with_different_arity() {
        let t1_src = "person(alice, 29)";
        let t2_src = "person(alice)";
        assert!(!parse_and_unify(t1_src, t2_src));
    }
}
