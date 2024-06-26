#![allow(non_snake_case)]

use assert2::{assert, check, let_assert};
use chumsky::Parser;
use test_log::test;

use crate::{
    cell::Cell,
    mem::Mem,
    syntax::Term,
    unify::{rec::unify, vm::Vm},
};

fn parse_and_unify_rec(t1_src: &str, t2_src: &str) -> bool {
    let mut mem = Mem::new();
    let t1 = tracing::trace_span!("parsing", src = t1_src)
        .in_scope(|| Term::parser().parse(t1_src).unwrap().serialize(&mut mem));
    let t2 = tracing::trace_span!("parsing", src = t2_src)
        .in_scope(|| Term::parser().parse(t2_src).unwrap().serialize(&mut mem));
    tracing::trace_span!(
        "rec unifying",
        t1 = %mem.display_term(t1),
        t2 = %mem.display_term(t2),
    )
    .in_scope(|| unify(&mut mem, t1, t2))
}

fn parse_and_unify_vm(t1_src: &str, t2_src: &str) -> bool {
    let mut mem = Mem::new();
    let t1 = tracing::trace_span!("parsing", src = t1_src)
        .in_scope(|| Term::parser().parse(t1_src).unwrap().serialize(&mut mem));
    let t2 = tracing::trace_span!("parsing", src = t2_src)
        .in_scope(|| Term::parser().parse(t2_src).unwrap().serialize(&mut mem));
    tracing::trace_span!(
        "vm unifying",
        t1 = %mem.display_term(t1),
        t2 = %mem.display_term(t2),
    )
    .in_scope(|| {
        let mut vm = Vm::new(mem);
        vm.setup_unification(t1, t2);
        vm.run_unification()
    })
}

#[test]
fn unify_ints() {
    let mut mem = Mem::new();
    let t1 = Term::Int(42).serialize(&mut mem);
    let t2 = Term::Int(42).serialize(&mut mem);
    assert!(unify(&mut mem, t1, t2));
}

#[test]
fn unify_syms() {
    let mut mem = Mem::new();
    let t1 = Term::Sym("socrates".into()).serialize(&mut mem);
    let t2 = Term::Sym("socrates".into()).serialize(&mut mem);
    check!(unify(&mut mem, t1, t2));

    let t3 = Term::Sym("aristotle".into()).serialize(&mut mem);
    check!(!unify(&mut mem, t1, t3));
}

#[test]
fn unify_identical_compound_terms() {
    let t1_src = "person(alice, 29)";
    let t2_src = "person(alice, 29)";
    check!(parse_and_unify_rec(t1_src, t2_src));
    check!(parse_and_unify_vm(t1_src, t2_src));
}

#[test]
fn unify_different_compound_terms() {
    let t1_src = "person(alice, 29)";
    let t2_src = "person(bob, 94)";
    check!(!parse_and_unify_rec(t1_src, t2_src));
    check!(!parse_and_unify_vm(t1_src, t2_src));
}

#[test]
fn unify_compound_terms_with_different_functors() {
    let t1_src = "person(alice, 29)";
    let t2_src = "inventory_item(adze, tool, weight(2, kg))";
    check!(!parse_and_unify_rec(t1_src, t2_src));
    check!(!parse_and_unify_vm(t1_src, t2_src));
}

#[test]
fn unify_compound_terms_with_different_arity() {
    let t1_src = "person(alice, 29)";
    let t2_src = "person(alice)";
    check!(!parse_and_unify_rec(t1_src, t2_src));
    check!(!parse_and_unify_vm(t1_src, t2_src));
}

#[test]
fn unify_vars() {
    check!(parse_and_unify_rec("A", "A"));
    check!(parse_and_unify_vm("A", "A"));
    check!(parse_and_unify_rec("A", "Z"));
    check!(parse_and_unify_vm("A", "Z"));
}

#[test]
fn unify_var_and_concrete() {
    check!(parse_and_unify_rec("X", "42"));
    check!(parse_and_unify_vm("X", "42"));

    check!(parse_and_unify_rec("f(X)", "f(42)"));
    check!(parse_and_unify_vm("f(X)", "f(42)"));

    check!(parse_and_unify_rec("f(X, 42)", "f(99, Y)"));
    check!(parse_and_unify_vm("f(X, 42)", "f(99, Y)"));
}

#[test]
fn test_unification_failure() {
    check!(!parse_and_unify_rec("f(X, 42)", "f(99, X)"));
    check!(!parse_and_unify_vm("f(X, 42)", "f(99, X)"));
}

#[test]
fn deep_unify() {
    let t1_src = "f(g(h(i(j(k(l(m(n(o(p(q(r(s(t(u(v(w(x(y(z))))))))))))))))))))";
    let t2_src = "f(g(h(i(j(k(l(m(n(o(p(q(r(s(t(u(v(w(x(y(Z))))))))))))))))))))";
    check!(parse_and_unify_rec(t1_src, t2_src));
    check!(parse_and_unify_vm(t1_src, t2_src));

    let t1_src = "f(g(h(i(j(k(l(m(n(o(p(q(r(s(t(u(v(w(x(y(z))))))))))))))))))))";
    let t2_src = "f(g(h(i(j(k(l(m(n(o(p(q(r(s(t(u(v(w(x(y(a))))))))))))))))))))";
    check!(!parse_and_unify_rec(t1_src, t2_src));
    check!(!parse_and_unify_vm(t1_src, t2_src));
}

#[test]
fn long_unify() {
    let t1_src = "f(g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z)";
    let t2_src = "f(g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, Z)";
    check!(parse_and_unify_rec(t1_src, t2_src));
    check!(parse_and_unify_vm(t1_src, t2_src));

    let t1_src = "f(g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z)";
    let t2_src = "f(g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, a)";
    check!(!parse_and_unify_rec(t1_src, t2_src));
    check!(!parse_and_unify_vm(t1_src, t2_src));
}

fn unify_rec(t1_src: &str, t2_src: &str, expect_unify_success: bool) -> Mem {
    tracing::trace_span!("REC").in_scope(|| {
        let mut mem = Mem::new();
        let t1 = Term::parser().parse(t1_src).unwrap().serialize(&mut mem);
        let t2 = Term::parser().parse(t2_src).unwrap().serialize(&mut mem);
        tracing::trace_span!("DO_UNIFY").in_scope(|| {
            assert!(unify(&mut mem, t1, t2) == expect_unify_success);
        });
        mem
    })
}

fn unify_vm(t1_src: &str, t2_src: &str, expect_unify_success: bool) -> Vm {
    tracing::trace_span!("VM").in_scope(|| {
        let mut mem = Mem::new();
        let t1 = Term::parser().parse(t1_src).unwrap().serialize(&mut mem);
        let t2 = Term::parser().parse(t2_src).unwrap().serialize(&mut mem);
        let mut vm = Vm::new(mem);
        vm.mem.assign_name_to_var(t1, "t1");
        vm.mem.assign_name_to_var(t2, "t2");
        tracing::trace_span!("DO_UNIFY").in_scope(|| {
            vm.setup_unification(t1, t2);
            assert!(vm.run_unification() == expect_unify_success);
        });
        vm
    })
}

#[test]
fn test_result_of_unification() {
    let t1_src = "f(X, 42)";
    let t2_src = "f(99, Y)";

    let mem = unify_rec(t1_src, t2_src, true);
    check!(mem.cell_from_var_name("X").unwrap() == Cell::Int(99));
    check!(mem.cell_from_var_name("Y").unwrap() == Cell::Int(42));

    // Now do it with the vm.
    let vm = unify_vm(t1_src, t2_src, true);
    check!(vm.mem.cell_from_var_name("X").unwrap() == Cell::Int(99));
    check!(vm.mem.cell_from_var_name("Y").unwrap() == Cell::Int(42));
}

#[test]
fn test_result_of_unification_complex__rec() {
    // let t1_src = "f(g(X), h(42))";
    // let t2_src = "f(g(99), h(Y))";
    let t1_src = "f(g(99), h(42))";
    let t2_src = "f(g(99), h(Y))";

    let mem = unify_rec(t1_src, t2_src, true);
    // check!(mem.cell_from_var_name("X").unwrap() == Cell::Int(99));
    check!(mem.cell_from_var_name("Y").unwrap() == Cell::Int(42));
    println!("REC MEM AFTER UNIFICATION");
    println!("{:?}", mem);
}

#[test]
fn test_result_of_unification_complex__vm() {
    let t1_src = "f(g(99), h(42))";
    let t2_src = "f(g(99), h(Y))";
    let vm = unify_vm(t1_src, t2_src, true);

    println!("VM MEM AFTER UNIFICATION");
    println!("{:?}", vm.mem);

    check!(vm.mem.cell_from_var_name("Y").unwrap() == Cell::Int(42));
}

#[test]
fn nested_pair_in_pair_with_var_at_cadr() {
    let t1_src = "f(777, g(X, 123))";
    let t2_src = "f(777, g(99, 123))";

    let mem = unify_rec(t1_src, t2_src, true);
    check!(mem.cell_from_var_name("X").unwrap() == Cell::Int(99));

    let vm = unify_vm(t1_src, t2_src, true);
    check!(vm.mem.cell_from_var_name("X").unwrap() == Cell::Int(99));
}

#[test]
fn nested_pair_in_pair_with_var_at_caddr() {
    let t1_src = "f(777, g(123, X))";
    let t2_src = "f(777, g(123, 99))";

    let mem = unify_rec(t1_src, t2_src, true);
    check!(mem.cell_from_var_name("X").unwrap() == Cell::Int(99));

    let vm = unify_vm(t1_src, t2_src, true);
    check!(vm.mem.cell_from_var_name("X").unwrap() == Cell::Int(99));
}

#[test]
fn nested_pair_in_pair_with_var_at_caar() {
    let t1_src = "f(g(X, 123), 777)";
    let t2_src = "f(g(99, 123), 777)";

    let mem = unify_rec(t1_src, t2_src, true);
    check!(mem.cell_from_var_name("X").unwrap() == Cell::Int(99));

    let vm = unify_vm(t1_src, t2_src, true);
    check!(vm.mem.cell_from_var_name("X").unwrap() == Cell::Int(99));
}

#[test]
fn nested_pair_in_pair_with_var_at_cadar() {
    let t1_src = "f(g(123, X), 777)";
    let t2_src = "f(g(123, 99), 777)";

    let mem = unify_rec(t1_src, t2_src, true);
    check!(mem.cell_from_var_name("X").unwrap() == Cell::Int(99));

    let vm = unify_vm(t1_src, t2_src, true);
    check!(vm.mem.cell_from_var_name("X").unwrap() == Cell::Int(99));
}

#[test]
fn nested_pair_var_in_1st_pos() {
    let t1_src = "f(g(X, 123))";
    let t2_src = "f(g(99, 123))";

    let mem = unify_rec(t1_src, t2_src, true);
    check!(mem.cell_from_var_name("X").unwrap() == Cell::Int(99));

    let vm = unify_vm(t1_src, t2_src, true);
    check!(vm.mem.cell_from_var_name("X").unwrap() == Cell::Int(99));
}

#[test]
fn nested_pair_var_in_2nd_pos() {
    let t1_src = "f(g(123, X))";
    let t2_src = "f(g(123, 99))";

    let mem = unify_rec(t1_src, t2_src, true);
    check!(mem.cell_from_var_name("X").unwrap() == Cell::Int(99));

    let vm = unify_vm(t1_src, t2_src, true);
    check!(vm.mem.cell_from_var_name("X").unwrap() == Cell::Int(99));
}

#[test]
fn nested_pair_vars_in_both_pos() {
    let t1_src = "f(g(123, X))";
    let t2_src = "f(g(Y, 99))";

    let mem = unify_rec(t1_src, t2_src, true);
    check!(mem.cell_from_var_name("X").unwrap() == Cell::Int(99));
    check!(mem.cell_from_var_name("Y").unwrap() == Cell::Int(123));

    let vm = unify_vm(t1_src, t2_src, true);
    check!(vm.mem.cell_from_var_name("X").unwrap() == Cell::Int(99));
    check!(vm.mem.cell_from_var_name("Y").unwrap() == Cell::Int(123));
}

#[test]
fn var_points_to_rcd() {
    let t1_src = "f(X)";
    let t2_src = "f(g(99))";

    {
        let mem = unify_rec(t1_src, t2_src, true);
        let_assert!(Some(cell) = mem.cell_from_var_name("X"));
        let_assert!(Cell::Rcd(r) = cell);
        let f = mem.resolve_ref_to_cell(r);
        let_assert!(Cell::Sig(f) = f);
        check!(f == mem.intern_functor("g", 1));
        let r = r + 1;
        let_assert!(Cell::Int(99) = mem.resolve_ref_to_cell(r));
    }

    {
        let vm = unify_vm(t1_src, t2_src, true);
        let_assert!(Some(cell) = vm.mem.cell_from_var_name("X"));
        let_assert!(Cell::Rcd(r) = cell);
        let f = vm.mem.resolve_ref_to_cell(r);
        let_assert!(Cell::Sig(f) = f);
        check!(f == vm.mem.intern_functor("g", 1));
        let r = r + 1;
        let_assert!(Cell::Int(99) = vm.mem.resolve_ref_to_cell(r));
    }
}
