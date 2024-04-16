use crate::{
    cell::Cell,
    defs::{CodeSeg, HeapSeg, Offset, StackSeg, Word},
    instr::{Reg, RegId},
};

pub mod exec;

pub const NREGS: usize = 16;

pub struct Vm {
    pub(crate) prog_ptr: Offset<CodeSeg>,
    pub(crate) cont_prog_ptr: Offset<CodeSeg>,
    pub(crate) last_env: Offset<StackSeg>,
    pub(crate) last_choice: Offset<StackSeg>,
    pub(crate) stack_top: usize,
    pub(crate) heap_top: usize,
    pub(crate) trail_top: usize,
    pub(crate) heap_backtrack: Offset<HeapSeg>,
    pub(crate) structure_ptr: Offset<HeapSeg>,
    pub(crate) regs: [Reg; NREGS],

    /// The code segment.
    pub(crate) code: Vec<Word>,
    pub(crate) heap: Vec<Word>,
    pub(crate) stack: Vec<Word>,
    pub(crate) trail: Vec<Word>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            prog_ptr: Offset::null(),
            cont_prog_ptr: Offset::null(),
            last_env: Offset::null(),
            last_choice: Offset::null(),
            stack_top: 0,
            heap_top: 0,
            trail_top: 0,
            heap_backtrack: Offset::null(),
            structure_ptr: Offset::null(),
            regs: [Reg::zero(); NREGS],
            code: Vec::new(),
            heap: Vec::new(),
            stack: Vec::new(),
            trail: Vec::new(),
        }
    }

    pub fn arg(&self, RegId(id): RegId) -> Word {
        (unsafe { self.regs[id as usize].arg }) as Word
    }

    pub fn set_arg(&mut self, RegId(id): RegId, value: Word) {
        self.regs[id as usize].arg = value as usize;
    }

    pub fn tmp(&self, RegId(id): RegId) -> Word {
        (unsafe { self.regs[id as usize].tmp }) as Word
    }

    pub fn set_tmp(&mut self, RegId(id): RegId, value: Word) {
        self.regs[id as usize].tmp = value as usize;
    }

    fn dereference(&self, arg: u32) -> Cell {
        todo!()
    }
}
