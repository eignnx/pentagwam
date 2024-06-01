//! Byte Code module.
#![allow(unused, clippy::useless_vec)]

use crate::{bc::vm::Vm, mem::Mem};

#[macro_use]
pub mod instr;
pub mod instr_fmt;
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
                    lbl: Some($lbl),
                    instr: $instr,
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
                    instr: $instr,
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
        Instr::PutStructure(foo_4, Arg(1));
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
        Instr::GetStructure(Arg(1), tree_3);
        Instr::UnifyVariable(Arg(2).into());
        Instr::UnifyVariable(Arg(3).into());
        Instr::UnifyVariable(Arg(4).into());
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
        Instr::SwitchOnTerm {
            on_var: c1a,
            on_const: c1,
            on_list: c2,
            on_struct: fail,
        };

        // Clause 1
        c1a:
            Instr::TryMeElse(c2a);
        c1:
            Instr::GetNil(Arg(1));
            Instr::GetValue(Arg(2).into(), Arg(3));
            Instr::Proceed;

        // Clause 2
        c2a:
            Instr::TrustMeElse(fail);
        c2:
            Instr::GetList(Arg(1));
            Instr::UnifyVariable(Reg(4).into());
            Instr::UnifyVariable(Arg(1).into());
            Instr::GetList(Arg(3));
            Instr::UnifyValue(Reg(4).into());
            Instr::UnifyVariable(Arg(3).into());
            Instr::Execute(concatenate_3);
    };

    println!("{:?}", bc);
}
