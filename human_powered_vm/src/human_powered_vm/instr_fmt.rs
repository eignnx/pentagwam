use core::fmt;

use pentagwam::{
    bc::instr::{Constant, Instr},
    cell::Functor,
    mem::Mem,
};

pub struct InstrFmt<'a, L> {
    instr: &'a Instr<L>,
    mem: &'a Mem,
}

pub fn display_instr<'a, L>(instr: &'a Instr<L>, mem: &'a Mem) -> InstrFmt<'a, L> {
    InstrFmt { instr, mem }
}

struct FmtConstant<'a> {
    constant: &'a Constant,
    mem: &'a Mem,
}

impl<'a> fmt::Display for FmtConstant<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.constant {
            Constant::Int(i) => write!(f, "{i:+}"),
            Constant::Sym(sym) => write!(f, "'{}'", sym.resolve(self.mem)),
        }
    }
}

struct FmtFunctor<'a> {
    functor: &'a Functor,
    mem: &'a Mem,
}

impl fmt::Display for FmtFunctor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}",
            self.functor.sym.resolve(self.mem),
            self.functor.arity
        )
    }
}

impl<'a, L: fmt::Debug> fmt::Display for InstrFmt<'a, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.instr {
            Instr::GetStructure(arg, functor) => {
                write!(
                    f,
                    "get_structure {}, {}",
                    arg,
                    FmtFunctor {
                        functor,
                        mem: self.mem
                    }
                )
            }
            Instr::UnifyVariable(slot) => write!(f, "unify_variable {}", slot),
            Instr::UnifyValue(slot) => write!(f, "unify_value {}", slot),
            Instr::GetVariable(slot, arg) => write!(f, "get_variable {}, {}", slot, arg),
            Instr::GetConst(slot, constant) => write!(
                f,
                "get_const {}, {}",
                slot,
                FmtConstant {
                    constant,
                    mem: self.mem
                }
            ),
            Instr::GetList(slot) => write!(f, "get_list {}", slot),
            Instr::GetNil(slot) => write!(f, "get_nil {}", slot),
            Instr::GetValue(slot, arg) => write!(f, "get_value {}, {}", slot, arg),
            Instr::GetVoid => write!(f, "get_void"),
            Instr::PutStructure(functor, arg) => {
                write!(
                    f,
                    "put_structure {}, {}",
                    FmtFunctor {
                        functor,
                        mem: self.mem
                    },
                    arg
                )
            }
            Instr::PutVariable(slot, arg) => write!(f, "put_variable {}, {}", slot, arg),
            Instr::PutValue { var_addr, arg } => write!(f, "put_value {}, {}", var_addr, arg),
            Instr::PutConst(constant, arg) => write!(
                f,
                "put_const {}, {}",
                FmtConstant {
                    constant,
                    mem: self.mem
                },
                arg
            ),
            Instr::PutList(arg) => write!(f, "put_list {}", arg),
            Instr::PutNil(arg) => write!(f, "put_nil {}", arg),
            Instr::PutVoid => write!(f, "put_void"),
            Instr::Call {
                functor,
                nvars_in_env,
            } => write!(
                f,
                "call {functor:?}, nvars={nvars_in_env}",
                // FmtFunctor {
                //     functor: todo!(),
                //     mem: self.mem
                // },
            ),
            Instr::Execute(functor) => write!(
                f,
                "execute {functor:?}",
                // FmtFunctor {
                //     functor: todo!(),
                //     mem: self.mem
                // }
            ),
            Instr::Proceed => write!(f, "proceed"),
            Instr::SwitchOnTerm {
                on_var,
                on_const,
                on_list,
                on_struct,
            } => write!(
                f,
                "switch_on_term var={on_var:?}, const={on_const:?}, \
                                list={on_list:?}, struct={on_struct:?}",
            ),
            Instr::TryMeElse(lbl) => write!(f, "try_me_else {:?}", lbl),
            Instr::TrustMeElse(lbl) => write!(f, "trust_me_else {:?}", lbl),
            Instr::SetVariable(_) => todo!(),
            Instr::SetValue(_) => todo!(),
            Instr::SetConstant(_) => todo!(),
        }
    }
}
