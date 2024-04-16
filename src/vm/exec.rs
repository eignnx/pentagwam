use crate::{
    cell::Cell,
    defs::Word,
    instr::{Get, Indexing, Instr, Procedural, Put, Unify},
};

use super::Vm;

impl Vm {
    pub fn exec(&mut self) {
        let word = self.code[self.prog_ptr.offset as usize];
        let opcode: u8 = (word >> (8 * 3)) as u8;
        let instr: Instr = Instr::Procedural(Procedural::Allocate); // TODO: decode instr
        match instr {
            Instr::Get { instr, src } => match instr {
                Get::VariableTmp { dst } => todo!(),
                Get::VariableLoc { dst } => todo!(),
                Get::ValueTmp { dst } => todo!(),
                Get::ValueLoc { dst } => todo!(),
                Get::Constant { val } => {
                    let arg = self.arg(src);
                    let tm: Cell = self.dereference(arg);
                }
                Get::Structure { f } => todo!(),
                Get::Nil => todo!(),
                Get::List => todo!(),
            },
            Instr::Put { instr, dst } => match instr {
                Put::VariableLoc { src } => {
                    let new = self.stack_top as Word + src.0 as Word;
                    self.set_arg(dst, new);
                    self.stack[self.stack_top - src.0 as usize] = new;
                }
                Put::VariableTmp { src } => todo!(),
                Put::ValueLoc { src } => todo!(),
                Put::ValueTmp { src } => todo!(),
                Put::UnsafeValue { src } => todo!(),
                Put::Constant { val } => todo!(),
                Put::Structure { f } => todo!(),
                Put::Nil => todo!(),
                Put::List => todo!(),
            },
            Instr::Unify(instr) => match instr {
                Unify::Void { nvars } => todo!(),
                Unify::VariableLoc { loc } => todo!(),
                Unify::VariableTmp { tmp } => todo!(),
                Unify::ValueLoc { loc } => todo!(),
                Unify::ValueTmp { tmp } => todo!(),
                Unify::LocalValueLoc { loc } => todo!(),
                Unify::LocalValueTmp { tmp } => todo!(),
                Unify::Constant { val } => todo!(),
                Unify::Nil => todo!(),
            },
            Instr::Procedural(instr) => match instr {
                Procedural::Proceed => todo!(),
                Procedural::Allocate => todo!(),
                Procedural::Deallocate => todo!(),
                Procedural::Execute { pred } => todo!(),
                Procedural::Call { pred, nvars } => todo!(),
            },
            Instr::Indexing(instr) => match instr {
                Indexing::TryMeElse { clause } => todo!(),
                Indexing::RetryMeElse { clause } => todo!(),
                Indexing::TrustMeElseFail => todo!(),
                Indexing::Try { clause } => todo!(),
                Indexing::Retry { clause } => todo!(),
                Indexing::Trust { clause } => todo!(),
                Indexing::SwitchOnTerm {
                    on_var,
                    on_const,
                    on_list,
                    on_struct,
                } => todo!(),
                Indexing::SwitchOnConstant { table, log2_n } => todo!(),
                Indexing::SwitchOnStructure { table, log2_n } => todo!(),
            },
        }
    }
}
