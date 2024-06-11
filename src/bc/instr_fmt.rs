use std::fmt;

use crate::{
    bc::instr::{Constant, Instr},
    cell::Functor,
    mem::{DisplayViaMem, Mem},
};

impl<S: DisplayViaMem> DisplayViaMem for Functor<S> {
    fn display_via_mem(&self, f: &mut core::fmt::Formatter<'_>, mem: &Mem) -> core::fmt::Result {
        write!(f, "{}/{}", mem.display(&self.sym), self.arity)
    }
}

impl<L, S> Instr<L, S> {
    pub fn instr_name(&self) -> &'static str {
        match self {
            Instr::SwitchOnTerm { .. } => "switch_on_term",
            Instr::TryMeElse(..) => "try_me_else",
            Instr::TrustMeElse(..) => "trust_me_else",
            Instr::Call { .. } => "call",
            Instr::Execute(..) => "execute",
            Instr::Proceed => "proceed",
            Instr::PutVariable(..) => "put_variable",
            Instr::PutValue { .. } => "put_value",
            Instr::PutConst(..) => "put_const",
            Instr::PutNil(..) => "put_nil",
            Instr::PutStructure(..) => "put_structure",
            Instr::PutList(..) => "put_list",
            Instr::GetConst(..) => "get_const",
            Instr::GetNil(..) => "get_nil",
            Instr::GetList(..) => "get_list",
            Instr::GetValue(..) => "get_value",
            Instr::GetVoid => "get_void",
            Instr::GetVariable(..) => "get_variable",
            Instr::GetStructure(..) => "get_structure",
            Instr::UnifyVariable(..) => "unify_variable",
            Instr::UnifyValue(..) => "unify_value",
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
