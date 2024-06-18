use std::fmt;

use crate::{
    bc::instr::{Constant, Instr},
    cell::Functor,
    mem::{DisplayViaMem, Mem},
};

use super::instr::InstrName;

impl<S: DisplayViaMem> DisplayViaMem for Functor<S> {
    fn display_via_mem(&self, f: &mut core::fmt::Formatter<'_>, mem: &Mem) -> core::fmt::Result {
        write!(f, "{}/{}", mem.display(&self.sym), self.arity)
    }
}

impl<L, S> Instr<L, S> {
    pub fn instr_name(&self) -> InstrName {
        match self {
            Instr::SwitchOnTerm { .. } => InstrName::SwitchOnTerm,
            Instr::TryMeElse(..) => InstrName::TryMeElse,
            Instr::TrustMeElse(..) => InstrName::TrustMeElse,
            Instr::Call { .. } => InstrName::Call,
            Instr::Execute(..) => InstrName::Execute,
            Instr::Proceed => InstrName::Proceed,
            Instr::PutVariable(..) => InstrName::PutVariable,
            Instr::PutValue { .. } => InstrName::PutValue,
            Instr::PutConst(..) => InstrName::PutConst,
            Instr::PutNil(..) => InstrName::PutNil,
            Instr::PutStructure(..) => InstrName::PutStructure,
            Instr::PutList(..) => InstrName::PutList,
            Instr::GetConst(..) => InstrName::GetConst,
            Instr::GetNil(..) => InstrName::GetNil,
            Instr::GetList(..) => InstrName::GetList,
            Instr::GetValue(..) => InstrName::GetValue,
            Instr::GetVoid => InstrName::GetVoid,
            Instr::GetVariable(..) => InstrName::GetVariable,
            Instr::GetStructure(..) => InstrName::GetStructure,
            Instr::UnifyVariable(..) => InstrName::UnifyVariable,
            Instr::UnifyValue(..) => InstrName::UnifyValue,
        }
    }
}

impl<L: fmt::Display, S: DisplayViaMem> DisplayViaMem for Instr<L, S> {
    fn display_via_mem(&self, f: &mut core::fmt::Formatter<'_>, mem: &Mem) -> core::fmt::Result {
        let name = self.instr_name();
        match self {
            Instr::GetStructure(arg, functor) => {
                write!(f, "{name} {}, {}", arg, mem.display(functor))
            }
            Instr::UnifyVariable(slot) => write!(f, "{name} {}", slot),
            Instr::UnifyValue(slot) => write!(f, "{name} {}", slot),
            Instr::GetVariable(slot, arg) => write!(f, "{name} {}, {}", slot, arg),
            Instr::GetConst(slot, constant) => {
                write!(f, "{name} {}, {}", slot, mem.display(constant))
            }
            Instr::GetList(slot) => write!(f, "{name} {}", slot),
            Instr::GetNil(slot) => write!(f, "{name} {}", slot),
            Instr::GetValue(slot, arg) => write!(f, "{name} {}, {}", slot, arg),
            Instr::GetVoid => write!(f, "{name}"),
            Instr::PutStructure(functor, arg) => {
                write!(f, "{name} {}, {}", mem.display(functor), arg)
            }
            Instr::PutVariable(slot, arg) => write!(f, "{name} {}, {}", slot, arg),
            Instr::PutValue { var_addr, arg } => write!(f, "{name} {}, {}", var_addr, arg),
            Instr::PutConst(constant, arg) => {
                write!(f, "{name} {}, {}", mem.display(constant), arg)
            }
            Instr::PutList(arg) => write!(f, "{name} {}", arg),
            Instr::PutNil(arg) => write!(f, "{name} {}", arg),
            Instr::Call { lbl, nvars_in_env } => write!(f, "{name} {lbl}, nvars={nvars_in_env}"),
            Instr::Execute(lbl) => write!(f, "{name} {lbl}"),
            Instr::Proceed => write!(f, "{name}"),
            Instr::SwitchOnTerm {
                on_var,
                on_const,
                on_list,
                on_struct,
            } => write!(
                f,
                "{name} var={on_var}, const={on_const}, \
                        list={on_list}, struct={on_struct}",
            ),
            Instr::TryMeElse(lbl) => write!(f, "{name} {lbl}"),
            Instr::TrustMeElse(lbl) => write!(f, "{name} {lbl}"),
        }
    }
}

impl fmt::Display for InstrName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use heck::ToSnakeCase;
        write!(f, "{}", format!("{self:?}").to_snake_case())
    }
}
