use crate::defs::{CodeSeg, DataSeg, Offset};

#[derive(Clone, Copy)]
pub(crate) union Reg {
    pub(crate) arg: usize,
    pub(crate) tmp: usize,
}

impl Reg {
    pub(crate) fn zero() -> Reg {
        Self { arg: 0 }
    }
}

pub(crate) enum Instr {
    Get { instr: Get, src: RegId },
    Put { instr: Put, dst: RegId },
    Unify(Unify),
    Procedural(Procedural),
    Indexing(Indexing),
}

/// The ID of a register.
pub(crate) struct RegId(pub(crate) u8);

/// The ID of a stack local variable.
pub(crate) struct LocId(pub(crate) u8);

pub(crate) struct Const(pub(crate) usize);

pub(crate) struct Functor(Offset<DataSeg>);

/// A get instruction which reads data from the argument registers.
pub(crate) enum Get {
    VariableTmp { dst: RegId },
    VariableLoc { dst: LocId },
    ValueTmp { dst: RegId },
    ValueLoc { dst: LocId },
    Constant { val: Const },
    Structure { f: Functor },
    Nil,
    List,
}

pub(crate) enum Put {
    /// Represents a goal argument that is an unbound (permenant) variable. The
    /// instruction puts a reference to permanent variable `Yn` into the
    /// register `Ai`, and also initializes `Yn` with the same reference.
    ///
    /// # Behavior
    /// ```asm
    /// vm.regs[dst].arg <- vm.stack_top + src
    /// STACK[vm.stack_top + src] <- vm.stack_top + src
    /// ```
    VariableLoc {
        src: LocId,
    },
    /// Represents an argument of the final goal that is an unbound variable.
    /// The instruction creates an unbound variable on the heap, and puts a
    /// reference to it into registers `Ai` and `Xn`.
    ///
    /// # Behavior
    /// ```asm
    /// let r = tag_ref(vm.heap_top) // WTF is tag_ref??
    /// next_term(vm.heap_top) <- r
    /// vm.regs[dst].tmp       <- r
    /// vm.regs[src].arg       <- r
    /// ```
    VariableTmp {
        src: RegId,
    },
    ValueLoc {
        src: LocId,
    },
    ValueTmp {
        src: RegId,
    },
    UnsafeValue {
        src: LocId,
    },
    Constant {
        val: Const,
    },
    Structure {
        f: Functor,
    },
    Nil,
    List,
}

pub(crate) enum Unify {
    /// A sequence of `nvars` singleton variables which may be skipped.
    Void {
        nvars: u8,
    },
    VariableLoc {
        loc: LocId,
    },
    VariableTmp {
        tmp: RegId,
    },
    ValueLoc {
        loc: LocId,
    },
    ValueTmp {
        tmp: RegId,
    },
    LocalValueLoc {
        loc: LocId,
    },
    LocalValueTmp {
        tmp: RegId,
    },
    Constant {
        val: Const,
    },
    Nil,
}

pub(crate) enum Procedural {
    Proceed,
    Allocate,
    Deallocate,
    Execute {
        pred: Offset<CodeSeg>,
    },
    Call {
        pred: Offset<CodeSeg>,
        /// The number of variables still in use.
        nvars: u8,
    },
}

pub(crate) enum Indexing {
    TryMeElse {
        clause: Offset<CodeSeg>,
    },
    RetryMeElse {
        clause: Offset<CodeSeg>,
    },
    TrustMeElseFail,
    Try {
        clause: Offset<CodeSeg>,
    },
    Retry {
        clause: Offset<CodeSeg>,
    },
    Trust {
        clause: Offset<CodeSeg>,
    },
    /// This instruction provides access to a group of clauses with a
    /// non-variable in the first head argument. It causes a dispatch on the
    /// type of the first arguemnt of the call. The argument `ArgId(1)` is
    /// dereferenced and, depending on whether the result is a variable,
    /// constant, (non-empty) list, or structure, the program pointer `P` is set
    /// to `Lv`, `Lc`, `Ll`, or `Ls`, respectively.
    SwitchOnTerm {
        on_var: Offset<CodeSeg>,
        on_const: Offset<CodeSeg>,
        on_list: Offset<CodeSeg>,
        on_struct: Offset<CodeSeg>,
    },
    /// This instruction provides hash table access to a group of clauses having
    /// constants in the first head argument position. Register `ArgId(1)` holds
    /// a constant, whose value is hashed to compute an index in the range
    /// `0..=N-1` into the hash table `Table`. The size of the hash table is
    /// `N`, which is a power of 2. The hash table entry gives access to the
    /// clause or clauses whose keys hash to that index. The constant in
    /// `ArgId(1)` is compared with the different keys until one is found that
    /// is identical, at which point the program pointer `P` is set to point to
    /// the corresponding clause or clauses. If the key is not found,
    /// backtracking occurs.
    SwitchOnConstant {
        table: Offset<CodeSeg>,
        /// The size of the table is `2^log2_n`.
        log2_n: u8,
    },
    SwitchOnStructure {
        table: Offset<CodeSeg>,
        /// The size of the table is `2^log2_n`.
        log2_n: u8,
    },
}
