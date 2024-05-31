use chumsky::{primitive::end, Parser};
use pentagwam::{bc::instr::Instr, cell::Functor};

use crate::human_powered_vm::{instr_fmt, vals::rval::RVal};

use super::{error::Result, HumanPoweredVm};

impl HumanPoweredVm {
    pub(super) fn program_listing(&self, rest: &[&str], program: &[Instr<Functor>]) -> Result<()> {
        match rest {
            [] => {
                for (i, instr) in program.iter().enumerate() {
                    println!("{:04}: {}", i, instr_fmt::display_instr(instr, &self.mem));
                }
            }
            ["from", n] => {
                let n = n.parse()?;
                for (i, instr) in program.iter().enumerate().skip(n) {
                    println!("{:04}: {}", i, instr_fmt::display_instr(instr, &self.mem));
                }
            }
            ["first", n] => {
                let n = n.parse()?;
                for (i, instr) in program.iter().enumerate().take(n) {
                    println!("{:04}: {}", i, instr_fmt::display_instr(instr, &self.mem));
                }
            }
            ["next", n] => {
                let n = n.parse()?;
                for (i, instr) in program.iter().enumerate().skip(self.instr_ptr).take(n) {
                    println!("{:04}: {}", i, instr_fmt::display_instr(instr, &self.mem));
                }
            }
            ["last", n] => {
                let n = n.parse()?;
                let skip = program.len().saturating_sub(n);
                for (i, instr) in program.iter().enumerate().skip(skip) {
                    println!("{:04}: {}", i, instr_fmt::display_instr(instr, &self.mem));
                }
            }
            ["prev" | "previous", n] => {
                let n = n.parse()?;
                let skip = self.instr_ptr.saturating_sub(n);
                for (i, instr) in program.iter().enumerate().skip(skip).take(n) {
                    println!("{:04}: {}", i, instr_fmt::display_instr(instr, &self.mem));
                }
            }
            _ => println!("!> Unknown `list` sub-command `{}`.", rest.join(" ")),
        }
        Ok(())
    }

    pub(super) fn print_rval(&self, rval_name: &str) -> Result<()> {
        let rval = RVal::parser().then_ignore(end()).parse(rval_name)?;
        let val = self.eval_to_val(&rval)?;
        println!(
            "=> {} == {}",
            rval.display(&self.mem),
            val.display(&self.mem),
        );
        Ok(())
    }

    pub(super) fn assign_to_lval(&mut self, lval_name: &str, rhs_name: &str) -> Result<()> {
        let lval = lval_name.parse()?;
        let rval = rhs_name.parse()?;
        let val = self.lval_set(&lval, &rval)?;
        println!(
            "Wrote `{}` to `{}`.",
            val.display(&self.mem),
            lval.display(&self.mem),
        );
        Ok(())
    }
}
