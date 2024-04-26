use derive_more::From;

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
    todo!()
}

pub fn try_me_else(lbl: Lbl) -> LabeledInstr {
    todo!()
}

pub fn get_nil(arg: Arg) -> LabeledInstr {
    todo!()
}

pub fn get_value(slot: impl Into<Slot>, arg: Arg) -> LabeledInstr {
    todo!()
}

pub fn proceed() -> LabeledInstr {
    todo!()
}

pub fn trust_me_else(lbl: Lbl) -> LabeledInstr {
    todo!()
}

pub fn get_list(arg: Arg) -> LabeledInstr {
    todo!()
}

pub fn unify_variable(slot: impl Into<Slot>) -> LabeledInstr {
    todo!()
}

pub fn unify_value(slot: impl Into<Slot>) -> LabeledInstr {
    todo!()
}

pub fn execute(lbl: Lbl) -> LabeledInstr {
    todo!()
}
