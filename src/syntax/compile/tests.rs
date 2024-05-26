use crate::bc::instr::Local;

use super::*;

fn var(name: &str) -> Term {
    Term::Var(Some(name.to_owned()))
}

fn binop(op: &str, lhs: Term, rhs: Term) -> Term {
    Term::Record(op.to_owned(), vec![lhs, rhs])
}

#[test]
fn compile_derivative_ex() {
    // d(U*V, X, DU*V + U*DV) :-
    //     d(U, X, DU),
    //     d(V, X, DV).

    let clause = Clause {
        head: (
            "d".to_owned(),
            vec![
                binop("*", var("U"), var("V")),
                var("X"),
                binop(
                    "+",
                    binop("*", var("DU"), var("V")),
                    binop("*", var("U"), var("DV")),
                ),
            ],
        ),
        body: vec![
            Term::Record("d".to_owned(), vec![var("U"), var("X"), var("DU")]),
            Term::Record("d".to_owned(), vec![var("V"), var("X"), var("DV")]),
        ],
    };

    let mut state = CompilerState::default();
    let mut out = Vec::new();
    state.compile_clause(&clause, &mut out).unwrap();

    let star2 = Functor {
        sym: state.intern_symbol("*"),
        arity: 2,
    };
    let plus2 = Functor {
        sym: state.intern_symbol("+"),
        arity: 2,
    };
    let d = Functor {
        sym: state.intern_symbol("d"),
        arity: 3,
    };

    let expected: Vec<LabelledInstr> = vec![
        Instr::GetStructure(Arg(1), star2),
        Instr::UnifyVariable(Slot::arg(1)),
        Instr::UnifyVariable(Slot::local(1)),
        Instr::GetVariable(Slot::local(2), Arg(2)),
        Instr::GetStructure(Arg(4), plus2),
        Instr::UnifyVariable(Slot::reg(4)),
        Instr::UnifyVariable(Slot::reg(5)),
        Instr::GetStructure(/*Reg*/ Arg(4), star2),
        Instr::UnifyVariable(Slot::arg(3)),
        Instr::UnifyValue(Slot::local(1)),
        Instr::GetStructure(/*Reg*/ Arg(5), star2),
        Instr::UnifyValue(Slot::arg(1)),
        Instr::UnifyVariable(Slot::local(3)),
        Instr::Call {
            functor: d,
            nvars_in_env: 3,
        },
        Instr::PutValue {
            var_addr: Local(1),
            arg: Arg(1),
        },
        Instr::PutValue {
            var_addr: Local(2),
            arg: Arg(2),
        },
        Instr::PutValue {
            var_addr: Local(3),
            arg: Arg(3),
        },
        Instr::Execute(d),
    ]
    .into_iter()
    .map(|i| todo!())
    .collect();

    assert_eq!(out, expected);
}
