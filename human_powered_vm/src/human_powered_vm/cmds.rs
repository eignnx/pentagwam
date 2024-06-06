use crate::human_powered_vm::{
    error::Error,
    vals::{lval::LVal, rval::RVal},
};

use super::{
    error::Result,
    vals::{slice::Region, val::Val},
    HumanPoweredVm,
};

impl HumanPoweredVm {
    pub(super) fn print_rval(&self, rval: &RVal) -> Result<()> {
        let val = self.eval_to_val(rval)?;
        if let Val::Slice { region, start, len } = val {
            self.print_slice(region, start, len)?;
        } else {
            println!("=> {}", self.mem.display(&val));
        }
        Ok(())
    }

    pub(super) fn print_slice(&self, region: Region, start: usize, len: usize) -> Result<()> {
        match region {
            Region::Mem => {
                println!("{:-^20}", "HEAP SEGMENT");
                for i in start..start + len {
                    let cell = self
                        .mem
                        .heap
                        .get(i)
                        .ok_or(Error::OutOfBoundsMemRead(region, i))?;
                    println!("{i:04}: {}", self.mem.display(cell));
                }
                println!("{:-^20}", "");
            }
            Region::Code => {
                println!("{:-^20}", "CODE SEGMENT");
                for i in start..start + len {
                    let instr = self
                        .program
                        .get(i)
                        .ok_or(Error::OutOfBoundsMemRead(region, i))?;
                    println!("{i:04}: {}", self.mem.display(instr));
                }
                println!("{:-^20}", "");
            }
        }
        Ok(())
    }

    pub(super) fn assign_to_lval(&mut self, lval_name: &str, rhs_name: &str) -> Result<()> {
        let lval: LVal = lval_name.parse()?;
        let rval: RVal = rhs_name.parse()?;
        let val = self.lval_set(&lval, &rval)?;
        println!(
            "Wrote `{}` to `{}`.",
            self.mem.display(&val),
            self.mem.display(&lval)
        );
        Ok(())
    }
}
