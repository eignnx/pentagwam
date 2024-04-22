use std::task::Poll;

use crate::{cell::Cell, defs::CellRef, mem::Mem};

pub struct Vm {
    mem: Mem,
    /// The worklist of items left to unify.
    stack: Vec<(CellRef, CellRef)>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            mem: Mem::new(),
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

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

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

    fn parse_and_unify_rec(t1_src: &str, t2_src: &str) -> bool {
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

    fn parse_and_unify_vm(t1_src: &str, t2_src: &str) -> bool {
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
        .in_scope(|| {
            let mut vm = Vm {
                mem,
                stack: Vec::new(),
            };
            vm.setup_unification(t1, t2);
            vm.run_unification()
        })
    }

    #[test]
    fn unify_identical_compound_terms() {
        let t1_src = "person(alice, 29)";
        let t2_src = "person(alice, 29)";
        assert!(parse_and_unify_rec(t1_src, t2_src));
        assert!(parse_and_unify_vm(t1_src, t2_src));
    }

    #[test]
    fn unify_different_compound_terms() {
        let t1_src = "person(alice, 29)";
        let t2_src = "person(bob, 94)";
        assert!(!parse_and_unify_rec(t1_src, t2_src));
        assert!(!parse_and_unify_vm(t1_src, t2_src));
    }

    #[test]
    fn unify_compound_terms_with_different_functors() {
        let t1_src = "person(alice, 29)";
        let t2_src = "inventory_item(adze, tool, weight(2, kg))";
        assert!(!parse_and_unify_rec(t1_src, t2_src));
        assert!(!parse_and_unify_vm(t1_src, t2_src));
    }

    #[test]
    fn unify_compound_terms_with_different_arity() {
        let t1_src = "person(alice, 29)";
        let t2_src = "person(alice)";
        assert!(!parse_and_unify_rec(t1_src, t2_src));
        assert!(!parse_and_unify_vm(t1_src, t2_src));
    }

    #[test]
    fn unify_vars() {
        assert!(parse_and_unify_rec("A", "A"));
        assert!(parse_and_unify_vm("A", "A"));
        assert!(parse_and_unify_rec("A", "Z"));
        assert!(parse_and_unify_vm("A", "Z"));
    }

    #[test]
    fn unify_var_and_concrete() {
        assert!(parse_and_unify_rec("X", "42"));
        assert!(parse_and_unify_vm("X", "42"));

        assert!(parse_and_unify_rec("f(X)", "f(42)"));
        assert!(parse_and_unify_vm("f(X)", "f(42)"));

        assert!(parse_and_unify_rec("f(X, 42)", "f(99, Y)"));
        assert!(parse_and_unify_vm("f(X, 42)", "f(99, Y)"));
    }

    #[test]
    fn test_unification_failure() {
        assert!(!parse_and_unify_rec("f(X, 42)", "f(99, X)"));
        assert!(!parse_and_unify_vm("f(X, 42)", "f(99, X)"));
    }
}
