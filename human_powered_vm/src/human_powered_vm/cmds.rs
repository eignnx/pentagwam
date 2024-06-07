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
            bunt::println!("=> {[yellow]}", self.mem.display(&val));
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
                    bunt::println!(
                        "{$dimmed}{:04}:{/$} {[blue+intense]}",
                        i,
                        self.mem.display(cell)
                    );
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
                    bunt::println!(
                        "{$dimmed}{:04}:{/$} {[intense]}",
                        i,
                        self.mem.display(instr)
                    );
                }
                println!("{:-^20}", "");
            }
        }
        Ok(())
    }

    pub(super) fn assign_to_lval(&mut self, lval_name: &str, rhs_name: &str) -> Result<()> {
        let lval: LVal = lval_name.parse()?;
        let rval: RVal = rhs_name.parse()?;
        let _val = self.lval_set(&lval, &rval)?;
        Ok(())
    }

    pub(super) fn add_alias(&mut self, new_name: &str, old_name: &str) -> Result<()> {
        // First check if the old name is for a temporary variable.
        if let Some(old_name) = old_name.strip_prefix('.') {
            // Ensure the new name also looks like a temporary variable.
            let Some(new_name) = new_name.strip_prefix('.') else {
                bunt::println!(
                    "{$red}!>{/$} An alias to a temporary variable must begin with a dot."
                );
                return Ok(());
            };

            // Check that the alias doesn't already exist.
            if let Some(existing_name) = self
                .tmp_vars
                .iter()
                .find_map(|(name, fdata)| fdata.aliases.contains(new_name).then_some(name))
            {
                bunt::println!(
                        "{$red}!>{/$} Alias `{$cyan}.{}{/$}` already exists for temporary variable `{$cyan}.{}{/$}`.",
                        new_name,
                        existing_name,
                    );
                return Ok(());
            }

            // Add alias.
            if let Some(fdata) = self.tmp_vars.get_mut(old_name) {
                fdata.aliases.insert(new_name.to_string());
                bunt::println!(
                    "Aliased temporary variable `{$cyan}.{old_name}{/$}` as `{$cyan}.{new_name}{/$}`.",
                    old_name = old_name,
                    new_name = new_name,
                );
            } else {
                bunt::println!(
                    "{$red}!>{/$} Can't alias `{$cyan+dimmed}.{old_name}{/$}` as `{$cyan+dimmed}.{new_name}{/$}` because \
                     temporary variable `{$cyan+dimmed}.{old_name}{/$}` doesn't exist.",
                    old_name = old_name,
                    new_name = new_name,
                );
            }
        } else if let Some(fdata) = self.fields.get_mut(old_name) {
            fdata.aliases.insert(new_name.to_string());
            bunt::println!(
                "Aliased field `{[cyan]old_name}` as `{[cyan]new_name}`.",
                old_name = old_name,
                new_name = new_name,
            );
        } else {
            bunt::println!(
                "{$red}!>{/$} Can't alias `{[cyan+dimmed]old_name}` as `{[cyan+dimmed]new_name}` \
                 because field `{[cyan+dimmed]old_name}` doesn't exist.",
                old_name = old_name,
                new_name = new_name,
            );
        }
        Ok(())
    }

    pub(super) fn delete_name(&mut self, name: &str) {
        todo!()
    }
}
