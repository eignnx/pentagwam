//! Byte Code module.
#![allow(unused, clippy::useless_vec)]

use crate::mem::Mem;

#[macro_use]
mod instr;
mod vm;

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
                LabeledInstr {
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
                LabeledInstr {
                    lbl: None,
                    instr: $instr.instr
                };
            ]
        )
    };
}

#[test]
fn push_struct_arg() {
    use instr::Arg::*;
    use instr::Reg::*;
    use instr::*;
    // let syntax = "foo(Y,abc,123,Y)".parse::<Term>().unwrap();

    let mut mem = Mem::new();

    let foo_4 = mem.intern_functor("foo", 4);
    let abc = mem.intern_sym("abc");

    let bc = wam_code! {
        put_structure(A1, foo_4);
        set_variable(A3);
        set_constant(abc);
        set_constant(123);
        set_value(A3);
    };
}

#[test]
fn concatenate_example() {
    use instr::Arg::*;
    use instr::Reg::*;
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
            get_nil(A1);
            get_value(A2, A3);
            proceed();

        // Clause 2
        c2a:
            trust_me_else(fail);
        c2:
            get_list(A1);
            unify_variable(X4);
            unify_variable(A1);
            get_list(A3);
            unify_value(X4);
            unify_variable(A3);
            execute(concatenate_3);
    };

    println!("{:?}", bc);
}
