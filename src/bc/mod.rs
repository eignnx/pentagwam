//! Byte Code module.
#![allow(unused, clippy::useless_vec)]

#[macro_use]
mod instr;
mod vm;

#[test]
fn use_bc_fns() {
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

    let bc = vec![
        lbl!(concatenate_3: switch_on_term(c1a, c1, c2, fail)),
        // Clause 1
        lbl!(c1a: try_me_else(c2a)),
        lbl!(c1: get_nil(A1)),
        get_value(A2, A3),
        proceed(),
        // Clause 2
        lbl!(c2a: trust_me_else(fail)),
        lbl!(c2: get_list(A1)),
        unify_variable(X4),
        unify_variable(A1),
        get_list(A3),
        unify_value(X4),
        unify_variable(A3),
        execute(concatenate_3),
    ];

    println!("{:?}", bc);
}
