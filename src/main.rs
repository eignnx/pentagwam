use defs::{CodeSeg, Offset};
use instr::{Get, Indexing, Instr, Procedural, RegId, Unify};
use vm::Vm;

mod cell;
mod defs;
mod instr;
mod vm;

fn main() {
    //     let vm = Vm {
    //         code: vec![Instr::Procedural(Proce)],
    //         prog_ptr: todo!(),
    //         cont_prog_ptr: todo!(),
    //         last_env: todo!(),
    //         last_choice: todo!(),
    //         stack_top: todo!(),
    //         heap_top: todo!(),
    //         trail_top: todo!(),
    //         heap_backtrack: todo!(),
    //         structure_ptr: todo!(),
    //         regs: todo!(),
    //         heap: todo!(),
    //         stack: todo!(),
    //         trail: todo!(),
    //     };
}

const APPEND_3: Offset<CodeSeg> = Offset::at(0);
const C1A: Offset<CodeSeg> = Offset::at(1);
const C1: Offset<CodeSeg> = Offset::at(2);
const C2: Offset<CodeSeg> = Offset::at(0);
const C2A: Offset<CodeSeg> = Offset::at(0);
const FAIL: Offset<CodeSeg> = Offset::at(0);

const APPEND_EX: &[Instr] = &[
    // append/3:
    Instr::Indexing(Indexing::SwitchOnTerm {
        on_var: C1A,
        on_const: C1,
        on_list: C2,
        on_struct: FAIL,
    }),
    // C1a:
    Instr::Indexing(Indexing::TryMeElse { clause: C2A }),
    // C1:
    Instr::Get {
        instr: Get::Nil,
        src: RegId(1),
    },
    Instr::Get {
        instr: Get::ValueTmp { dst: RegId(2) },
        src: RegId(3),
    },
    Instr::Procedural(Procedural::Proceed),
    // C2a:
    Instr::Indexing(Indexing::TrustMeElseFail),
    // C2:
    Instr::Get {
        instr: Get::List,
        src: RegId(1),
    },
    Instr::Unify(Unify::VariableTmp { tmp: RegId(4) }),
    Instr::Unify(Unify::VariableTmp { tmp: RegId(1) }),
    Instr::Get {
        instr: Get::List,
        src: RegId(3),
    },
    Instr::Unify(Unify::ValueTmp { tmp: RegId(4) }),
    Instr::Unify(Unify::VariableTmp { tmp: RegId(3) }),
    Instr::Procedural(Procedural::Execute { pred: APPEND_3 }),
];
