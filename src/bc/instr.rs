use core::fmt;

use derive_more::From;

use crate::{cell::Functor, defs::Sym};

/// A unique identifier for a label.
pub type Lbl = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, From)]
pub struct Reg(pub u8);

impl From<Arg> for Reg {
    fn from(arg: Arg) -> Self {
        Self(arg.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, From)]
pub struct Arg(pub u8);

impl From<Reg> for Arg {
    fn from(reg: Reg) -> Self {
        Self(reg.0)
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, From)]
pub struct Local(pub u16);

impl fmt::Display for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Y{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, From)]
pub enum Slot {
    #[from]
    Reg(Reg),
    #[from]
    Local(Local),
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Slot::Reg(reg) => write!(f, "X{}", reg.0),
            Slot::Local(local) => write!(f, "Y{}", local.0),
        }
    }
}

impl From<Arg> for Slot {
    fn from(value: Arg) -> Self {
        Self::Reg(Reg(value.0))
    }
}

impl Slot {
    pub fn reg(r: impl Into<Reg>) -> Self {
        r.into().into()
    }

    pub fn arg(a: impl Into<Arg>) -> Self {
        a.into().into()
    }

    pub fn local(l: impl Into<Local>) -> Self {
        Self::Local(l.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelledInstr {
    pub lbl: Option<Lbl>,
    pub instr: Instr<Lbl>,
}

impl From<Instr<Lbl>> for LabelledInstr {
    fn from(instr: Instr<Lbl>) -> Self {
        Self { lbl: None, instr }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, documented::DocumentedVariants)]
pub enum Instr<L = ()> {
    /// Dispatch on the type of the value pointed to by `X1`.
    SwitchOnTerm {
        on_var: L,
        on_const: L,
        on_list: L,
        on_struct: L,
    },

    TryMeElse(L),

    Proceed,

    TrustMeElse(L),

    UnifyVariable(Slot),

    UnifyValue(Slot),

    Call {
        functor: L,
        /// The number of variables in the environment at this point. Accessed
        /// as an offset from `CP` by certain instructions in the called
        /// procedure.
        nvars_in_env: u8,
    },

    Execute(L),

    /// The `set_variable` instruction creates an unbound variable on
    /// the heap, and makes `slot` point to it.
    SetVariable(Slot),

    /// The `set_value` instruction copies the value in `slot` to the heap.
    SetValue(Slot),

    SetConstant(Constant),

    GetNil(Arg),

    GetConst(Arg, Constant),

    GetList(Arg),

    GetValue(Slot, Arg),

    /// Tells the thing to skip to next argument since no processing is required
    /// for anonymous variables.
    GetVoid,

    GetVariable(Slot, Arg),

    /// The `get_structure` instruction starts by dereferencing A1
    /// and checking whether it is free
    /// - If it is free, it sets the current mode to `Mode::Write`. This makes
    ///   the rest of the `get_structure` behave like `put_structure`, and it
    ///   makes the subsequent `unify_variable` instructions behave like
    ///   `set_variable`.
    /// - If it is bound, it sets the current mode to `Mode::Read`. This makes
    ///   the rest of the `get_structure` and the subsequent `unify_variable`
    ///   instructions do matching against the existing term, instead of
    ///   constructing a new one.
    GetStructure(Arg, Functor),

    /// When the `Slot` is a stack-slot:
    /// - This instruction a goal argument that is an unbound (permenant)
    ///   variable. The instruction puts a reference to the permanent variable
    ///   into the register, and also initializes the slot with the same
    ///   reference.
    /// When the `Slot` is a register:
    /// - This instruction represents an argument of the final goal that is an
    /// unbound variable. The instruction creates an unbound variable on the
    /// heap, and puts a reference to it into the `Slot` and the `Arg`.
    PutVariable(Slot, Arg),

    /// This instruction represents a goal argument that is a bound variable.
    /// The instruction simply puts the value of variable `var_addr` into the
    /// register `arg`.
    PutValue {
        var_addr: Local,
        arg: Arg,
    },

    /// This instruction simply puts the constant into the `Arg` register.
    PutConst(Constant, Arg),

    /// This instruction simply puts the constant `[]` into the `Arg` register.
    PutNil(Arg),

    PutVoid,

    /// The `put_structure` instruction allocates only the cell header,
    /// the word pointing to the cell and the word identifying the
    /// function symbol. The arguments have to be filled in by the
    /// following instructions, each of which allocates one word on
    /// the heap.
    ///
    /// The instruction pushes the functor for the structure onto the heap, and
    /// puts a corresponding structure pointer into the `Arg` register.
    /// Execution then proceeds in *write* mode.
    PutStructure(Functor, Arg),

    /// This instruction marks the beginning of a list occurring as a goal
    /// argument. The instruction places a list pointer corresponding to the top
    /// of the heap into the `Arg` register. Execution then proceeds in *write*
    /// mode.
    PutList(Arg),
}

impl<L: Clone> Instr<L> {
    /// The crate [`documented`](https://github.com/cyqsimon/documented) (as of
    /// version 0.4.1) doesn't work with an enum that has a generic type
    /// parameter. This is a workaround which involves setting the type `()` as
    /// a default parameter for `L` and discarding the `L` when calling
    /// `get_variant_docs`.
    pub fn doc_comment(&self) -> Option<&'static str> {
        let mapped: Instr<()> = self.clone().map_lbl(|_| ()); // Throw away `L`, make it `()`.
        documented::DocumentedVariants::get_variant_docs(&mapped).ok()
    }
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
            Instr::GetStructure(arg, functor) => Instr::GetStructure(arg, functor),
            Instr::GetConst(arg, constant) => Instr::GetConst(arg, constant),
            // Instr::GetVariable(slot) => Instr::GetVariable(slot),
            _ => todo!(),
        }
    }
}

pub fn switch_on_term(on_var: Lbl, on_const: Lbl, on_list: Lbl, on_struct: Lbl) -> LabelledInstr {
    Instr::SwitchOnTerm {
        on_var,
        on_const,
        on_list,
        on_struct,
    }
    .into()
}

pub fn try_me_else(lbl: Lbl) -> LabelledInstr {
    Instr::TryMeElse(lbl).into()
}

pub fn get_nil(arg: Arg) -> LabelledInstr {
    Instr::GetNil(arg).into()
}

pub fn get_value(slot: impl Into<Slot>, arg: Arg) -> LabelledInstr {
    Instr::GetValue(slot.into(), arg).into()
}

pub fn proceed() -> LabelledInstr {
    Instr::Proceed.into()
}

pub fn trust_me_else(lbl: Lbl) -> LabelledInstr {
    Instr::TrustMeElse(lbl).into()
}

pub fn get_list(arg: Arg) -> LabelledInstr {
    Instr::GetList(arg).into()
}

pub fn unify_variable(slot: impl Into<Slot>) -> LabelledInstr {
    Instr::UnifyVariable(slot.into()).into()
}

pub fn unify_value(slot: impl Into<Slot>) -> LabelledInstr {
    Instr::UnifyValue(slot.into()).into()
}

pub fn execute(lbl: Lbl) -> LabelledInstr {
    Instr::Execute(lbl).into()
}

pub fn put_structure(arg: Arg, functor: Functor) -> LabelledInstr {
    Instr::PutStructure(functor, arg).into()
}

pub fn set_variable(slot: impl Into<Slot>) -> LabelledInstr {
    Instr::SetVariable(slot.into()).into()
}

#[derive(Debug, Clone, Copy, From, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Constant {
    #[from]
    Sym(Sym),
    #[from]
    Int(i32),
}

pub fn set_constant(constant: impl Into<Constant>) -> LabelledInstr {
    Instr::SetConstant(constant.into()).into()
}

pub fn set_value(slot: impl Into<Slot>) -> LabelledInstr {
    Instr::SetValue(slot.into()).into()
}

pub fn get_structure(arg: Arg, functor: Functor) -> LabelledInstr {
    Instr::GetStructure(arg, functor).into()
}
