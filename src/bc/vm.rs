use std::collections::HashMap;

use crate::{
    cell::Cell,
    defs::{CellRef, Sym},
    mem::Mem,
};

use super::instr::{Instr, LabelledInstr, Lbl, Reg, Slot};

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

pub const NREGS: usize = 16;

pub struct Vm {
    /// Program counter. Points to an instruction in `self.code`.
    pc: u32,
    regs: [CellRef; NREGS],
    mem: Mem,
    code: Vec<Instr<u32>>,
    choices: Vec<u32>,
    /// Pointer to the current structure being processed. Points into
    /// `self.mem.heap`.
    structure_ptr: CellRef,
    mode: Option<Mode>,
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Read,
    Write,
}

impl Vm {
    pub fn new(mem: Mem) -> Self {
        Self {
            pc: 0,
            regs: [CellRef::default(); NREGS],
            mem,
            code: Vec::new(),
            choices: Vec::new(),
            structure_ptr: 0.into(),
            mode: None,
        }
    }

    pub fn with_code(mut self, code: Vec<LabelledInstr>) -> Self {
        let mut labels: HashMap<Lbl, u32> = HashMap::new();

        for (i, instr) in code.iter().enumerate() {
            if let Some(lbl) = instr.lbl {
                labels.insert(lbl, i as u32);
            }
        }

        self.code = code
            .into_iter()
            .map(|instr| instr.instr.map_lbl(|lbl| labels[&lbl]))
            .collect();

        self
    }

    pub fn with_entry(mut self, entry: u32) -> Self {
        self.pc = entry;
        self
    }

    #[track_caller]
    fn fail(&mut self) {
        self.pc = self.choices.pop().unwrap();
    }

    fn reg(&self, reg: impl Into<Reg>) -> CellRef {
        self.regs[reg.into().0 as usize]
    }

    fn reg_mut(&mut self, reg: impl Into<Reg>) -> &mut CellRef {
        &mut self.regs[reg.into().0 as usize]
    }

    pub fn step(&mut self) -> Result<()> {
        match self.code[self.pc as usize] {
            Instr::SwitchOnTerm {
                on_var,
                on_const,
                on_list,
                on_struct,
            } => {
                match self.mem.resolve_ref_to_cell(self.regs[0]) {
                    Cell::Ref(_) => self.pc = on_var,
                    Cell::Int(_) | Cell::Sym(_) | Cell::Sig(_) => self.pc = on_const,
                    Cell::Lst(_) | Cell::Nil => self.pc = on_list,
                    Cell::Rcd(_) => self.pc = on_struct,
                }
                Ok(())
            }
            Instr::GetNil(arg) => {
                match self.mem.resolve_ref_to_cell(self.regs[arg.0 as usize]) {
                    Cell::Ref(var_ref) => {
                        self.mem.cell_write(var_ref, Cell::Nil);
                        self.pc += 1;
                    }
                    Cell::Nil => self.pc += 1,
                    _ => self.fail(),
                }
                Ok(())
            }
            Instr::GetList(arg) => {
                match self.mem.resolve_ref_to_cell(self.regs[arg.0 as usize]) {
                    Cell::Ref(var_ref) => {
                        let car_ref = self.mem.push_fresh_var();
                        let _cdr_ref = self.mem.push_fresh_var();
                        self.mem.cell_write(var_ref, Cell::Lst(car_ref));
                        // TODO: SAVE OLD VALUE OF `var_ref` TO TRAIL
                        *self.reg_mut(arg) = car_ref;
                        // self.regs[arg.0 as usize] = car_ref;
                        self.mode = Some(Mode::Write);
                        self.pc += 1;
                    }
                    Cell::Lst(r) => {
                        self.structure_ptr = r;
                        self.mode = Some(Mode::Read);
                        self.pc += 1;
                    }
                    _ => self.fail(),
                }
                Ok(())
            }
            Instr::TryMeElse(_) => todo!(),
            Instr::GetValue(_, _) => todo!(),
            Instr::Proceed => todo!(),
            Instr::TrustMeElse(_) => todo!(),
            Instr::UnifyVariable(_) => todo!(),
            Instr::UnifyValue(_) => todo!(),
            Instr::Execute(_) => todo!(),
            Instr::PutStructure(_, _) => todo!(),
            Instr::SetVariable(_) => todo!(),
            Instr::SetValue(_) => todo!(),
            Instr::SetConstant(_) => todo!(),
            Instr::GetStructure(_, _) => todo!(),
            // Instr::GetConst(_) => todo!(),
            _ => todo!(),
        }
    }

    fn slot_read(&self, slot: impl Into<Slot>) -> Result<Cell> {
        match slot.into() {
            Slot::Reg(r) => Ok(self.mem.resolve_ref_to_cell(self.regs[r.0 as usize])),
            Slot::Local(_) => todo!(),
        }
    }

    fn slot_write(&mut self, slot: impl Into<Slot>, cell_ref: CellRef) -> Result<()> {
        match slot.into() {
            Slot::Reg(r) => self.regs[r.0 as usize] = cell_ref,
            Slot::Local(_) => todo!(),
        }
        Ok(())
    }
}
