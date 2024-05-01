use derive_more::From;

use crate::{cell::Functor, defs::Sym};

/// A unique identifier for a label.
pub type Lbl = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reg {
    X1,
    X2,
    X3,
    X4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arg {
    A1,
    A2,
    A3,
    A4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Local(u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, From)]
pub enum Slot {
    #[from]
    Reg(Reg),
    #[from]
    Arg(Arg),
    #[from]
    Local(Local),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabeledInstr {
    pub lbl: Option<Lbl>,
    pub instr: Instr<Lbl>,
}

impl From<Instr<Lbl>> for LabeledInstr {
    fn from(instr: Instr<Lbl>) -> Self {
        Self { lbl: None, instr }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instr<L> {
    SwitchOnTerm {
        on_var: L,
        on_const: L,
        on_list: L,
        on_struct: L,
    },
    TryMeElse(L),
    GetNil(Arg),
    GetValue(Slot, Arg),
    Proceed,
    TrustMeElse(L),
    GetList(Arg),
    UnifyVariable(Slot),
    UnifyValue(Slot),
    Execute(L),
    PutStructure(Arg, Functor),
    SetVariable(Slot),
    SetValue(Slot),
    SetConstant(Constant),
}
impl<L> Instr<L> {
    pub fn map_lbl<M>(self, f: impl Fn(L) -> M) -> Instr<M> {
        match self {
            Instr::SwitchOnTerm {
                on_var,
                on_const,
                on_list,
                on_struct,
            } => Instr::SwitchOnTerm {
                on_var: f(on_var),
                on_const: f(on_const),
                on_list: f(on_list),
                on_struct: f(on_struct),
            },
            Instr::TryMeElse(lbl) => Instr::TryMeElse(f(lbl)),
            Instr::GetNil(arg) => Instr::GetNil(arg),
            Instr::GetValue(slot, arg) => Instr::GetValue(slot, arg),
            Instr::Proceed => Instr::Proceed,
            Instr::TrustMeElse(lbl) => Instr::TrustMeElse(f(lbl)),
            Instr::GetList(arg) => Instr::GetList(arg),
            Instr::UnifyVariable(slot) => Instr::UnifyVariable(slot),
            Instr::UnifyValue(slot) => Instr::UnifyValue(slot),
            Instr::Execute(lbl) => Instr::Execute(f(lbl)),
            Instr::PutStructure(arg, functor) => Instr::PutStructure(arg, functor),
            Instr::SetVariable(slot) => Instr::SetVariable(slot),
            Instr::SetValue(slot) => Instr::SetValue(slot),
            Instr::SetConstant(constant) => Instr::SetConstant(constant),
        }
    }
}

/// Labels the instruction.
macro_rules! lbl {
    ($l:ident : $instr:expr) => {
        LabeledInstr {
            lbl: Some($l),
            instr: $instr.instr,
        }
    };
}

pub fn switch_on_term(on_var: Lbl, on_const: Lbl, on_list: Lbl, on_struct: Lbl) -> LabeledInstr {
    Instr::SwitchOnTerm {
        on_var,
        on_const,
        on_list,
        on_struct,
    }
    .into()
}

pub fn try_me_else(lbl: Lbl) -> LabeledInstr {
    Instr::TryMeElse(lbl).into()
}

pub fn get_nil(arg: Arg) -> LabeledInstr {
    Instr::GetNil(arg).into()
}

pub fn get_value(slot: impl Into<Slot>, arg: Arg) -> LabeledInstr {
    Instr::GetValue(slot.into(), arg).into()
}

pub fn proceed() -> LabeledInstr {
    Instr::Proceed.into()
}

pub fn trust_me_else(lbl: Lbl) -> LabeledInstr {
    Instr::TrustMeElse(lbl).into()
}

pub fn get_list(arg: Arg) -> LabeledInstr {
    Instr::GetList(arg).into()
}

pub fn unify_variable(slot: impl Into<Slot>) -> LabeledInstr {
    Instr::UnifyVariable(slot.into()).into()
}

pub fn unify_value(slot: impl Into<Slot>) -> LabeledInstr {
    Instr::UnifyValue(slot.into()).into()
}

pub fn execute(lbl: Lbl) -> LabeledInstr {
    Instr::Execute(lbl).into()
}

pub fn put_structure(arg: Arg, functor: Functor) -> LabeledInstr {
    Instr::PutStructure(arg, functor).into()
}

pub fn set_variable(slot: impl Into<Slot>) -> LabeledInstr {
    Instr::SetVariable(slot.into()).into()
}

#[derive(Debug, Clone, Copy, From, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Constant {
    #[from]
    Sym(Sym),
    #[from]
    Int(i32),
}

pub fn set_constant(constant: impl Into<Constant>) -> LabeledInstr {
    Instr::SetConstant(constant.into()).into()
}

pub fn set_value(slot: impl Into<Slot>) -> LabeledInstr {
    Instr::SetValue(slot.into()).into()
}
