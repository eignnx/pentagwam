use crate::{cell::Cell, defs::Idx, mem::Mem};

pub fn unify(mem: &mut Mem, t1_idx: Idx, t2_idx: Idx) -> bool {
    let (t1_idx, t1) = mem.follow_refs_with_idx(t1_idx);
    let (t2_idx, t2) = mem.follow_refs_with_idx(t2_idx);

    // Step 1: ensure cell types match.
    match (t1, t2) {
        (Cell::Sig(f1), Cell::Sig(f2)) => {
            tracing::warn!("why are we unifying functors? {f1:?} {f2:?}");
            f1 == f2
        }
        (Cell::Int(i1), Cell::Int(i2)) => i1 == i2,
        (Cell::Sym(s1), Cell::Sym(s2)) => s1 == s2,
        // Two unbound variables:
        (Cell::Ref(idx1), Cell::Ref(idx2)) => {
            tracing::trace!(
                "unifying var `{}` and var `{}`",
                mem.human_readable_var_name(idx1),
                mem.human_readable_var_name(idx2),
            );
            // Make t1 point to t2 (arbitrary choice).
            mem.cell_write(t1_idx, Cell::Ref(t2_idx));
            // TODO: record variable binding in trail.
            true
        }
        (Cell::Ref(idx), concrete) => {
            tracing::trace!(
                "unifying var `{}` and concrete `{}`",
                mem.human_readable_var_name(idx),
                mem.display_cell(concrete),
            );
            // Make the var point to the concrete value.
            mem.cell_write(t1_idx, Cell::Ref(t2_idx));
            // TODO: record variable binding in trail.
            true
        }
        (concrete, Cell::Ref(idx)) => {
            tracing::trace!(
                "unifying concrete `{}` and var `{}`",
                mem.display_cell(concrete),
                mem.human_readable_var_name(idx),
            );
            // Make the var point to the concrete value.
            mem.cell_write(t2_idx, Cell::Ref(t1_idx));
            // TODO: record variable binding in trail.
            true
        }
        (Cell::Rcd(f_idx1), Cell::Rcd(f_idx2)) => {
            // Step 2: ensure functors match.

            let Cell::Sig(f1) = mem.cell_read(f_idx1) else {
                tracing::warn!("expected functor cell at index {f_idx1}");
                return false;
            };

            let Cell::Sig(f2) = mem.cell_read(f_idx2) else {
                tracing::warn!("expected functor cell at index {f_idx2}");
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
            let base1 = f_idx1 + 1;
            let base2 = f_idx2 + 1;

            // Step 3: unify arguments.
            for i in 0..f1.arity as usize {
                let arg1_idx = base1 + i;
                let arg2_idx = base2 + i;
                if !unify(mem, arg1_idx, arg2_idx) {
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
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;
    use crate::syntax::Syntax;
    use chumsky::Parser;

    #[test]
    fn unify_ints() {
        let mut mem = Mem::new();
        let t1 = Syntax::Int(42).serialize(&mut mem);
        let t2 = Syntax::Int(42).serialize(&mut mem);
        assert!(unify(&mut mem, t1, t2));
    }

    #[test]
    fn unify_syms() {
        let mut mem = Mem::new();
        let t1 = Syntax::Sym("socrates".into()).serialize(&mut mem);
        let t2 = Syntax::Sym("socrates".into()).serialize(&mut mem);
        assert!(unify(&mut mem, t1, t2));

        let t3 = Syntax::Sym("aristotle".into()).serialize(&mut mem);
        assert!(!unify(&mut mem, t1, t3));
    }

    #[track_caller]
    fn parse_and_unify(t1_src: &str, t2_src: &str) -> bool {
        let mut mem = Mem::new();
        let t1 = tracing::trace_span!("parsing", src = t1_src)
            .in_scope(|| Syntax::parser().parse(t1_src).unwrap().serialize(&mut mem));
        let t2 = tracing::trace_span!("parsing", src = t2_src)
            .in_scope(|| Syntax::parser().parse(t2_src).unwrap().serialize(&mut mem));
        tracing::trace_span!(
            "unifying",
            t1 = %mem.display_term(t1),
            t2 = %mem.display_term(t2),
        )
        .in_scope(|| unify(&mut mem, t1, t2))
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

    #[test]
    fn unify_vars() {
        assert!(parse_and_unify("X", "X"));
        assert!(parse_and_unify("X", "Y"));
    }

    #[test]
    fn unify_var_and_concrete() {
        assert!(parse_and_unify("X", "42"));
        assert!(parse_and_unify("f(X)", "f(42)"));
    }

    #[test]
    fn test_unification_failure() {
        assert!(!parse_and_unify("f(X, 42)", "f(99, X)"));
    }
}
