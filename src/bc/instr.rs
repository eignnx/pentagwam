use core::fmt;

use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::{cell::Functor, defs::Sym};

/// A unique identifier for a label.
pub type Lbl = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Reg(pub u8);

impl From<Arg> for Reg {
    fn from(arg: Arg) -> Self {
        Self(arg.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Serialize, Deserialize)]
#[serde(transparent)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Local(pub u16);

impl fmt::Display for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Y{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, documented::DocumentedVariants, Serialize, Deserialize)]
pub enum Instr<L> {
    /// # switch_on_term Lv, Lc, Ll, Ls
    /// This instruction provides access to a group of clauses with a non-
    /// variable in the first head argument. It causes a dispatch on the type
    /// of the first argument of the call. The argument A1 is dereferenced and,
    /// depending on whether the result is a variable, constant, (non-empty)
    /// list, or structure, the program pointer P is set to Lv, Lc, Ll, or Ls,
    /// respectively.
    SwitchOnTerm {
        on_var: L,
        on_const: L,
        on_list: L,
        on_struct: L,
    },

    /// # try_me_else L
    /// This instruction precedes the code for the first clause in a
    /// procedure with more than one clause. A choice point is created by
    /// saving the following n+8 values on the stack: registers An through A1,
    /// the current environment pointer E, the current continuation CP, a
    /// pointer to the previous choice point B, the address L of the next
    /// clause, the current trail pointer TR, and the current heap pointer H.
    /// HB is set to the current heap pointer, and B is set to point to the
    /// current top of stack.
    TryMeElse(L),

    /// # trust_me_else fail
    /// This instruction precedes the code for the last clause in a procedure.
    /// (The argument of the instruction is arbitrary, but exists simply to
    /// reserve space in the instruction in order to facilitate the asserting
    /// and retracting of clauses). The current choice point is discarded,
    /// and registers B and HB are reset to correspond to the previous choice
    /// point.
    ///
    ///     B := B(B)
    ///     HB := H(B)
    ///
    TrustMeElse(L),

    /// This instruction terminates a body goal and is responsible for
    /// setting CP to the following code, and the program pointer P to the
    /// procedure. N is the number of variables in the environment at this
    /// point. It is accessed as an offset from CP by certain instructions in
    /// the called procedure.
    ///
    ///     CP := following code
    ///     P := Proc
    ///
    Call {
        functor: L,
        /// The number of variables in the environment at this point. Accessed
        /// as an offset from `CP` by certain instructions in the called
        /// procedure.
        nvars_in_env: u8,
    },

    /// This instruction terminates the final goal in the body of a clause.
    /// The program pointer P is set to point to the procedure.
    ///
    ///     P := Proc
    ///
    Execute(L),

    /// This instruction terminates a unit clause. The program pointer P is
    /// reset to the continuation pointer CP.
    ///
    ///     P := CP
    ///
    Proceed,

    /// # put_variable Yn,Ai
    /// This instruction represents a goal argument that is an unbound
    /// (permanent) variable. The instruction puts a reference to permanent
    /// variable Yn into the register Ai, and also initializes Yn with the
    /// same reference.
    ///
    ///     Ai := Yn := ref_to(Yn)
    ///
    /// # put_variable Xn, Ai
    /// This instruction represents an argument of the final goal that is an
    /// unbound variable. The instruction creates an unbound variable on the
    /// heap, and puts a reference to it into registers Ai and Xn.
    ///
    ///     Ai := Xn := next_term(H) := tag_ref(H)
    ///
    /// # Alternate Explanation
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

    /// # put_value Va, Ai
    /// This instruction represents a goal argument that is a bound variable.
    /// The instruction simply puts the value of variable Vn into the register
    /// Ai.
    ///
    ///     Ai := Va
    ///
    /// # Alternate Explanation
    /// This instruction represents a goal argument that is a bound variable.
    /// The instruction simply puts the value of variable `var_addr` into the
    /// register `arg`.
    PutValue { var_addr: Local, arg: Arg },

    /// # put_const C, Ai
    /// This instruction represents a goal argument that is a constant. The instruction simply puts the constant C into register Ai.
    ///
    ///     Ai := C
    ///
    PutConst(Constant, Arg),

    /// This instruction simply puts the constant `[]` into the `Arg` register.
    PutNil(Arg),

    /// # put_structure F, Ai
    /// This instruction marks the beginning of a structure (without embedded
    /// substructures) occurring as a goal argument. The instruction pushes the
    /// functor F for the structure onto the heap, and puts a corresponding
    /// structure pointer into register Ai. Execution then proceeds in “write"
    /// mode.
    ///
    ///     Ai := tag_struct(H)
    ///     next_term(H) := F
    ///
    /// # Alternate Explanation
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

    /// # put_list Ai
    /// This instruction marks the beginning of a list occurring as a goal
    /// argument. The instruction places a list pointer corresponding to the top
    /// of the heap into register Ai. Execution then proceeds in "write" mode.
    ///     Ai = tag_list(H)
    ///
    /// # Alternate Explanation
    /// This instruction marks the beginning of a list occurring as a goal
    /// argument. The instruction places a list pointer corresponding to the top
    /// of the heap into the `Arg` register. Execution then proceeds in *write*
    /// mode.
    PutList(Arg),

    // /// The `set_variable` instruction creates an unbound variable on
    // /// the heap, and makes `slot` point to it.
    // SetVariable(Slot),
    //
    // /// The `set_value` instruction copies the value in `slot` to the heap.
    // SetValue(Slot),
    //
    // SetConstant(Constant),
    /// This instruction represents a head argument that is a constant. The
    /// instruction gets the value of register Ai and dereferences it. If the
    /// result is a reference to a variable, that variable is bound to the
    /// constant C, and the binding is trailed if necessary. Otherwise, the
    /// result is compared with the constant C, and if the two values are not
    /// identical, backtracking occurs.
    GetConst(Arg, Constant),

    /// This instruction represents a head argument that is the constant `[]`.
    /// The instruction gets the value of register Ai and dereferences it. If
    /// the result is a reference to a variable, that variable is bound to
    /// the constant `[]`, and the binding is trailed if necessary. Otherwise,
    /// the result is compared with the constant `[]`, and if the two values
    /// are not identical, backtracking occurs.
    GetNil(Arg),

    /// This instruction marks the beginning of a list occurring as a head
    /// argument. The instruction gets the value of register Ai and
    /// dereferences it. If the result is a reference to a variable, that
    /// variable is bound to a new list pointer pointing at the top of the
    /// heap, the binding is trailed if necessary, and execution proceeds in
    /// "write" mode. Otherwise, if the result is a list, the pointer S is set
    /// to point to the arguments of the list, and execution proceeds in "read"
    /// mode. Otherwise, backtracking occurs.
    GetList(Arg),

    /// This instruction represents a head argument that is a bound variable.
    /// The instruction gets the value of register Ai and unifies it with the
    /// contents of variable Va. The fully dereferenced result of the
    /// unification is left in variable Vn if Vn is a temporary.
    GetValue(Slot, Arg),

    /// Tells the thing to skip to next argument since no processing is required
    /// for anonymous variables.
    GetVoid,

    /// This instruction represents a head argument that is an unbound variable.
    /// The instruction simply gets the value of register Ai and stores it in
    /// variable Vn.
    ///     Vn := Ai
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
    ///
    /// # Alternate Explanation
    /// This instruction marks the beginning of a structure (without embedded
    /// substructures) occurring as a head argument. The instruction gets the
    /// value of register Ai and dereferences it. If the result is a
    /// reference to a variable, that variable is bound to a new structure
    /// pointer pointing at the top of the heap, and the binding is trailed
    /// if necessary, functor F is pushed onto the heap, and execution
    /// proceeds in "write" mode. Otherwise, if the result is a structure and
    /// its functor is identical to functor F, the pointer S is set to point
    /// to the arguments of the structure, and execution proceeds in "read"
    /// mode. Otherwise, backtracking occurs.
    GetStructure(Arg, Functor),

    /// # unify_variable Vn
    /// This instruction represents a head structure argument that is an
    /// unbound variable. If the instruction is executed in "read" mode, it
    /// simply gets the next argument from S and stores it in variable Vn. If
    /// the instruction is executed in "write" mode, it pushes a new unbound
    /// variable onto the heap, and stores a reference to it in variable Vn.
    ///
    /// In read mode:
    ///
    ///     Vn := next_term(S)
    ///
    /// In write mode:
    ///
    ///     Vn := next_term(H) = tag_ref(H)
    ///
    UnifyVariable(Slot),

    /// # unify_value Vn
    /// This instruction represents a head structure argument that is a
    /// variable bound to some global value. If the instruction is executed
    /// in "read" mode, it gets the next argument from S, and unifies it with
    /// the value in variable Vn, leaving the dereferenced result in Vn if Vn
    /// is a temporary. If the instruction is executed in "write" mode, it
    /// pushes the value of variable Vn onto the heap.
    ///
    /// In write mode:
    ///     next_term(H) := Vn
    UnifyValue(Slot),
}

impl<L> Instr<L> {
    pub fn doc_comment(&self) -> Option<&'static str> {
        documented::DocumentedVariants::get_variant_docs(self).ok()
    }

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
            Instr::GetStructure(arg, functor) => Instr::GetStructure(arg, functor),
            Instr::GetConst(arg, constant) => Instr::GetConst(arg, constant),
            Instr::Call {
                functor,
                nvars_in_env,
            } => Instr::Call {
                functor: f(functor),
                nvars_in_env,
            },
            Instr::GetVoid => Instr::GetVoid,
            Instr::GetVariable(slot, arg) => Instr::GetVariable(slot, arg),
            Instr::PutVariable(slot, arg) => Instr::PutVariable(slot, arg),
            Instr::PutValue { var_addr, arg } => Instr::PutValue { var_addr, arg },
            Instr::PutConst(konst, arg) => Instr::PutConst(konst, arg),
            Instr::PutNil(arg) => Instr::PutNil(arg),
            Instr::PutList(arg) => Instr::PutList(arg),
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

#[derive(
    Debug, Clone, Copy, From, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Constant {
    #[from]
    Sym(Sym),
    #[from]
    Int(i32),
}

pub fn get_structure(arg: Arg, functor: Functor) -> LabelledInstr {
    Instr::GetStructure(arg, functor).into()
}
