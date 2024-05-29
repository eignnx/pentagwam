//! Byte Code module.
#![allow(unused, clippy::useless_vec)]

use crate::{bc::vm::Vm, mem::Mem};

#[macro_use]
pub mod instr;
pub mod vm;

macro_rules! wam_code {
    ($($stuff:tt)*) => {
        wam_code_impl!([$($stuff)*] => [])
    };
}

macro_rules! wam_code_impl {
    ([] => [$($finished:expr;)*]) => {
        vec![$($finished),*]
    };

    ([$lbl:ident : $instr:expr; $($stuff:tt)*] => [$($finished:expr;)*]) => {
        wam_code_impl!(
            [$($stuff)*] =>
            [
                $($finished;)*
                LabelledInstr {
                    lbl:Some($lbl),
                    instr: $instr.instr
                };
            ]
        )
    };

    ([$instr:expr; $($stuff:tt)*] => [$($finished:expr;)*]) => {
        wam_code_impl!(
            [$($stuff)*] => [
                $($finished;)*
                LabelledInstr {
                    lbl: None,
                    instr: $instr.instr
                };
            ]
        )
    };
}

#[test]
fn push_struct_arg() {
    use instr::Arg;
    use instr::Reg;
    use instr::*;
    // let syntax = "foo(Y,abc,123,Y)".parse::<Term>().unwrap();

    let mut mem = Mem::new();

    let foo_4 = mem.intern_functor("foo", 4);
    let abc = mem.intern_sym("abc");

    let bc = wam_code! {
        put_structure(Arg(1), foo_4);
        // set_variable(Arg(3));
        // set_constant(abc);
        // set_constant(123);
        // set_value(Arg(3));
    };
}

#[test]
fn inside_clause() {
    use instr::Arg;
    use instr::Reg;
    use instr::*;

    let mut mem = Mem::new();

    let tree_3 = mem.intern_functor("tree", 3);

    // p(tree(X,L,R)) :- …
    let bc = wam_code! {
        get_structure(Arg(1), tree_3);
        unify_variable(Arg(2));
        unify_variable(Arg(3));
        unify_variable(Arg(4));
    };

    Vm::new(mem).with_code(bc).step();
}

#[test]
fn concatenate_example() {
    use instr::Arg;
    use instr::Reg;
    use instr::*;

    let mut lbl_id = 0;

    let mut fresh_lbl = || {
        let old = lbl_id;
        lbl_id += 1;
        old
    };

    let concatenate_3 = fresh_lbl();
    let c1a = fresh_lbl();
    let c1 = fresh_lbl();
    let c2 = fresh_lbl();
    let fail = fresh_lbl();
    let c2a = fresh_lbl();

    let bc = wam_code! {
        concatenate_3:
            switch_on_term(c1a, c1, c2, fail);

        // Clause 1
        c1a:
            try_me_else(c2a);
        c1:
            get_nil(Arg(1));
            get_value(Arg(2), Arg(3));
            proceed();

        // Clause 2
        c2a:
            trust_me_else(fail);
        c2:
            get_list(Arg(1));
            unify_variable(Reg(4));
            unify_variable(Arg(1));
            get_list(Arg(3));
            unify_value(Reg(4));
            unify_variable(Arg(3));
            execute(concatenate_3);
    };

    println!("{:?}", bc);
}
