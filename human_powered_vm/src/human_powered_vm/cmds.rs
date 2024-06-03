use chumsky::{primitive::end, Parser};

use crate::human_powered_vm::vals::{lval::LVal, rval::RVal};

use super::{
    error::{Error, Result},
    vals::val::{Region, Val},
    HumanPoweredVm,
};

impl HumanPoweredVm {
    pub(super) fn program_listing(&self, rest: &[&str]) -> Result<()> {
        match rest {
            [] => {
                for (i, instr) in self.program.iter().enumerate() {
                    println!("{:04}: {}", i, self.mem.display(instr));
                }
            }
            ["from", n] => {
                let n = n.parse()?;
                for (i, instr) in self.program.iter().enumerate().skip(n) {
                    println!("{:04}: {}", i, self.mem.display(instr));
                }
            }
            ["first", n] => {
                let n = n.parse()?;
                for (i, instr) in self.program.iter().enumerate().take(n) {
                    println!("{:04}: {}", i, self.mem.display(instr));
                }
            }
            ["next", n] => {
                let n = n.parse()?;
                for (i, instr) in self
                    .program
                    .iter()
                    .enumerate()
                    .skip(self.instr_ptr())
                    .take(n)
                {
                    println!("{:04}: {}", i, self.mem.display(instr));
                }
            }
            ["last", n] => {
                let n = n.parse()?;
                let skip = self.program.len().saturating_sub(n);
                for (i, instr) in self.program.iter().enumerate().skip(skip) {
                    println!("{:04}: {}", i, self.mem.display(instr));
                }
            }
            ["prev" | "previous", n] => {
                let n = n.parse()?;
                let skip = self.instr_ptr().saturating_sub(n);
                for (i, instr) in self.program.iter().enumerate().skip(skip).take(n) {
                    println!("{:04}: {}", i, self.mem.display(instr));
                }
            }
            _ => println!("!> Unknown `list` sub-command `{}`.", rest.join(" ")),
        }
        Ok(())
    }

    pub(super) fn print_rval(&self, rval_name: &str) -> Result<()> {
        let rval = RVal::parser().then_ignore(end()).parse(rval_name)?;
        let val = self.eval_to_val(&rval)?;
        if let Val::Slice { region, start, len } = val {
            match region {
                Region::Mem => {
                    let old_len = self.mem.heap.len();
                    let start = start.unwrap_or(0);
                    let len = match (old_len, len) {
                        (old_len, None) => old_len,
                        (old_len, Some(new)) if new <= old_len - start => new,
                        (old_len, Some(new_len)) => {
                            return Err(Error::BadSliceBounds {
                                old_len,
                                new_start: start,
                                new_len,
                            })
                        }
                    };
                    println!("{:^-20}", "HEAP SEGMENT");
                    for i in start..start + len {
                        println!("{i:04}: {}", self.mem.display(&self.mem.heap[i]));
                    }
                    println!("{:^-20}", "");
                }
                Region::Code => {
                    let old_len = self.program.len();
                    let start = start.unwrap_or(0);
                    let len = match (old_len, len) {
                        (old_len, None) => old_len,
                        (old_len, Some(new)) if new <= old_len - start => new,
                        (old_len, Some(new_len)) => {
                            return Err(Error::BadSliceBounds {
                                old_len,
                                new_start: start,
                                new_len,
                            })
                        }
                    };
                    println!("{:^-20}", "CODE SEGMENT");
                    for i in start..start + len {
                        println!("{i:04}: {}", self.mem.display(&self.program[i]));
                    }
                    println!("{:^-20}", "");
                }
            }
        } else {
            println!(
                "=> {} == {}",
                self.mem.display(&rval),
                self.mem.display(&val)
            );
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
