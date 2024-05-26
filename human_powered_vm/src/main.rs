use human_powered_vm::HumanPoweredVm;
use pentagwam::{
    bc::instr::{Arg, Instr, Local, Slot},
    cell::Functor,
};

pub mod human_powered_vm;

fn main() -> human_powered_vm::error::Result<()> {
    let mem = pentagwam::mem::Mem::new();

    let star2 = Functor {
        sym: mem.intern_sym("*"),
        arity: 2,
    };
    let plus2 = Functor {
        sym: mem.intern_sym("+"),
        arity: 2,
    };
    let d = Functor {
        sym: mem.intern_sym("d"),
        arity: 3,
    };

    let instrs = [
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
    .collect::<Vec<Instr<_>>>();

    let mut vm = HumanPoweredVm::new()?;
    vm.run(&instrs)?;

    Ok(())
}
