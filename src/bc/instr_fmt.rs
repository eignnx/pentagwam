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

impl<L: fmt::Display, S: DisplayViaMem> DisplayViaMem for Instr<L, S> {
    fn display_via_mem(&self, f: &mut core::fmt::Formatter<'_>, mem: &Mem) -> core::fmt::Result {
        match self {
            Instr::GetStructure(arg, functor) => {
                write!(f, "get_structure {}, {}", arg, mem.display(functor))
            }
            Instr::UnifyVariable(slot) => write!(f, "unify_variable {}", slot),
            Instr::UnifyValue(slot) => write!(f, "unify_value {}", slot),
            Instr::GetVariable(slot, arg) => write!(f, "get_variable {}, {}", slot, arg),
            Instr::GetConst(slot, constant) => {
                write!(f, "get_const {}, {}", slot, mem.display(constant))
            }
            Instr::GetList(slot) => write!(f, "get_list {}", slot),
            Instr::GetNil(slot) => write!(f, "get_nil {}", slot),
            Instr::GetValue(slot, arg) => write!(f, "get_value {}, {}", slot, arg),
            Instr::GetVoid => write!(f, "get_void"),
            Instr::PutStructure(functor, arg) => {
                write!(f, "put_structure {}, {}", mem.display(functor), arg)
            }
            Instr::PutVariable(slot, arg) => write!(f, "put_variable {}, {}", slot, arg),
            Instr::PutValue { var_addr, arg } => write!(f, "put_value {}, {}", var_addr, arg),
            Instr::PutConst(constant, arg) => {
                write!(f, "put_const {}, {}", mem.display(constant), arg)
            }
            Instr::PutList(arg) => write!(f, "put_list {}", arg),
            Instr::PutNil(arg) => write!(f, "put_nil {}", arg),
            Instr::Call { lbl, nvars_in_env } => write!(f, "call {lbl}, nvars={nvars_in_env}"),
            Instr::Execute(lbl) => write!(f, "execute {lbl}"),
            Instr::Proceed => write!(f, "proceed"),
            Instr::SwitchOnTerm {
                on_var,
                on_const,
                on_list,
                on_struct,
            } => write!(
                f,
                "switch_on_term var={on_var}, const={on_const}, \
                                list={on_list}, struct={on_struct}",
            ),
            Instr::TryMeElse(lbl) => write!(f, "try_me_else {lbl}"),
            Instr::TrustMeElse(lbl) => write!(f, "trust_me_else {lbl}"),
        }
    }
}
