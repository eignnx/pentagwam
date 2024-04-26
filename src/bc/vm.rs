use std::collections::HashMap;

use crate::{cell::Cell, mem::Mem};

use super::instr::{Instr, LabeledInstr, Lbl};

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

pub const NREGS: usize = 16;

pub struct Vm {
    pc: u32,
    regs: [Cell; NREGS],
    mem: Mem,
    code: Vec<Instr<u32>>,
}

impl Vm {
    pub fn new(mem: Mem) -> Self {
        Self {
            pc: 0,
            regs: [Cell::default(); NREGS],
            mem,
            code: Vec::new(),
        }
    }

    pub fn with_code(mut self, code: Vec<LabeledInstr>) -> Self {
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

    pub fn step(&mut self) -> Result<()> {
        match self.code[self.pc as usize] {
            Instr::SwitchOnTerm {
                on_var,
                on_const,
                on_list,
                on_struct,
            } => {
                match self.regs[0] {
                    Cell::Ref(_) => self.pc = on_var,
                    Cell::Rcd(_) => self.pc = on_struct,
                    Cell::Sym(_) | Cell::Sig(_) | Cell::Int(_) => self.pc = on_const,
                }
                Ok(())
            }
            Instr::TryMeElse(_) => todo!(),
            Instr::GetNil(_) => todo!(),
            Instr::GetValue(_, _) => todo!(),
            Instr::Proceed => todo!(),
            Instr::TrustMeElse(_) => todo!(),
            Instr::GetList(_) => todo!(),
            Instr::UnifyVariable(_) => todo!(),
            Instr::UnifyValue(_) => todo!(),
            Instr::Execute(_) => todo!(),
        }
    }
}
