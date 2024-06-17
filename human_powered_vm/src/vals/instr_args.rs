use pentagwam::{
    bc::instr::{Arg, Constant, Instr, Local, Reg, Slot},
    cell::Functor,
};

use crate::human_powered_vm::{
    error::{Error, Result},
    HumanPoweredVm,
};

use super::rval::RVal;

impl HumanPoweredVm {
    pub fn instr_param(&self, idx: usize) -> Result<RVal> {
        let instr = self
            .program
            .get(self.instr_ptr())
            .ok_or(Error::InstrPtrOutOfBounds(self.instr_ptr()))?;

        let params = instr_params(instr);

        let one_based_idx = idx.checked_sub(1).ok_or(Error::UndefinedInstrArg {
            param_idx: idx,
            param_count: params.len(),
        })?;

        params
            .get(one_based_idx)
            .cloned()
            .ok_or(Error::UndefinedInstrArg {
                param_idx: idx,
                param_count: params.len(),
            })
    }
}

pub fn instr_params(instr: &Instr<Functor<String>, String>) -> Vec<RVal> {
    match instr {
        Instr::SwitchOnTerm {
            on_var,
            on_const,
            on_list,
            on_struct,
        } => vec![
            on_var.into(),
            on_const.into(),
            on_list.into(),
            on_struct.into(),
        ],
        Instr::TryMeElse(lbl) => vec![lbl.into()],
        Instr::TrustMeElse(lbl) => vec![lbl.into()],
        Instr::Call { lbl, nvars_in_env } => {
            vec![lbl.into(), RVal::Usize(*nvars_in_env as usize)]
        }
        Instr::Execute(lbl) => vec![lbl.into()],
        Instr::Proceed => vec![],
        Instr::PutVariable(slot, arg) => vec![slot.into(), arg.into()],
        Instr::PutValue { var_addr, arg } => vec![var_addr.into(), arg.into()],
        Instr::PutConst(konst, arg) => vec![konst.into(), arg.into()],
        Instr::PutNil(arg) => vec![arg.into()],
        Instr::PutStructure(f, arg) => vec![f.into(), arg.into()],
        Instr::PutList(arg) => vec![arg.into()],
        Instr::GetConst(arg, konst) => vec![arg.into(), konst.into()],
        Instr::GetNil(arg) => vec![arg.into()],
        Instr::GetList(arg) => vec![arg.into()],
        Instr::GetValue(slot, arg) => vec![slot.into(), arg.into()],
        Instr::GetVoid => vec![],
        Instr::GetVariable(slot, arg) => vec![slot.into(), arg.into()],
        Instr::GetStructure(arg, f) => vec![arg.into(), f.into()],
        Instr::UnifyVariable(slot) => vec![slot.into()],
        Instr::UnifyValue(slot) => vec![slot.into()],
    }
}

impl From<&Slot> for RVal {
    fn from(value: &Slot) -> Self {
        match value {
            Slot::Reg(reg) => reg.into(),
            Slot::Local(loc) => loc.into(),
        }
    }
}

impl From<&Reg> for RVal {
    fn from(Reg(n): &Reg) -> Self {
        RVal::Field(format!("X{n}"))
    }
}

impl From<&Arg> for RVal {
    fn from(Arg(n): &Arg) -> Self {
        RVal::Field(format!("A{n}"))
    }
}

impl From<&Local> for RVal {
    fn from(Local(n): &Local) -> Self {
        RVal::Usize(*n as usize)
    }
}

impl From<&Constant<String>> for RVal {
    fn from(konst: &Constant<String>) -> Self {
        match konst {
            Constant::Sym(s) => RVal::Symbol(s.clone()),
            Constant::Int(i) => RVal::I32(*i),
        }
    }
}

impl From<&Functor<String>> for RVal {
    fn from(f: &Functor<String>) -> Self {
        RVal::Functor(
            Box::new(RVal::Symbol(f.sym.clone())),
            Box::new(RVal::Usize(f.arity as usize)),
        )
    }
}
