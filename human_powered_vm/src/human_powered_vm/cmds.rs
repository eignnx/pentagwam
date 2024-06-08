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
        if let Some(no_dot_old_name) = old_name.strip_prefix('.') {
            // Ensure the new name also looks like a temporary variable.
            let Some(no_dot_new_name) = new_name.strip_prefix('.') else {
                bunt::println!(
                    "{$red}!>{/$} An alias to a temporary variable must begin with a dot."
                );
                return Ok(());
            };

            // Check that the alias doesn't already exist.
            if let Some(existing_name) = self.tmp_vars.iter().find_map(|(name, fdata)| {
                fdata
                    .aliases
                    .contains(no_dot_new_name)
                    .then_some(format!(".{}", name))
            }) {
                bunt::println!(
                    "{$red}!>{/$} Cannot create alias `{[cyan+dimmed]new_name}` of temporary \
                    variable `{[cyan]old_name}` because `{[cyan]new_name}` already aliases \
                    temporary variable `{[cyan]existing_name}`.",
                    old_name = old_name,
                    new_name = new_name,
                    existing_name = existing_name,
                );
                return Ok(());
            }

            // Add alias.
            if let Some(fdata) = self.tmp_vars.get_mut(no_dot_old_name) {
                fdata.aliases.insert(no_dot_new_name.to_string());
                bunt::println!(
                    "Aliased temporary variable `{[cyan]old_name}` as \
                    `{[cyan]new_name}`.",
                    old_name = old_name,
                    new_name = new_name,
                );
            } else if let Some((tmp_var_name, fdata)) = self
                .tmp_vars
                .iter_mut()
                .find(|(_, fdata)| fdata.aliases.contains(no_dot_old_name))
            {
                // Actually add the alias under `field_name` because `old_name` aliases it.
                fdata.aliases.insert(no_dot_new_name.to_string());
                let tmp_var_name = format!(".{tmp_var_name}");
                bunt::println!(
                    "Aliased `{[cyan]old_name}` (temporary variable `{[cyan]tmp_var_name}`) as `{[cyan]new_name}`.",
                    old_name = old_name,
                    tmp_var_name = tmp_var_name,
                    new_name = new_name,
                );
            } else {
                bunt::println!(
                    "{$red}!>{/$} Can't alias `{[cyan+dimmed]old_name}` as \
                    `{[cyan+dimmed]new_name}` because temporary variable \
                    `{[cyan+dimmed]old_name}` doesn't exist.",
                    old_name = old_name,
                    new_name = new_name,
                );
            }
        } else {
            // Check that the alias doesn't already exist.
            if let Some(existing_name) = self
                .fields
                .iter()
                .find_map(|(name, fdata)| fdata.aliases.contains(new_name).then_some(name))
            {
                bunt::println!(
                    "{$red}!>{/$} Cannot create alias `{[cyan+dimmed]new_name}` of field \
                    `{[cyan]old_name}` because `{[cyan]new_name}` already aliases field \
                    `{[cyan]existing_name}`.",
                    old_name = old_name,
                    new_name = new_name,
                    existing_name = existing_name,
                );
                return Ok(());
            }

            if let Some(fdata) = self.fields.get_mut(old_name) {
                // Check that `new_name` doesn't begin with a dot.
                if new_name.starts_with('.') {
                    bunt::println!(
                        "{$red}!>{/$} An alias of a field cannot begin with a dot (`.`)."
                    );
                    return Ok(());
                }
                fdata.aliases.insert(new_name.to_string());
                bunt::println!(
                    "Aliased field `{[cyan]old_name}` as `{[cyan]new_name}`.",
                    old_name = old_name,
                    new_name = new_name,
                );
            } else if let Some((field_name, fdata)) = self
                .fields
                .iter_mut()
                .find(|(_, fdata)| fdata.aliases.contains(old_name))
            {
                // Actually add the alias under `field_name` because `old_name` aliases it.
                if new_name.starts_with('.') {
                    bunt::println!(
                        "{$red}!>{/$} An alias of a field cannot begin with a dot (`.`)."
                    );
                    return Ok(());
                }
                fdata.aliases.insert(new_name.to_string());
                bunt::println!(
                    "Aliased `{[cyan]old_name}` (field `{[cyan]field_name}`) as `{[cyan]new_name}`.",
                    old_name = old_name,
                    field_name = field_name,
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
        }
        Ok(())
    }

    pub(super) fn delete_name(&mut self, name: &str) -> Result<()> {
        // Several cases to consider:
        // - name refers to a temp var
        //   + temp var exists
        //   + alias to a temp var exists
        //   + error
        // - name refers to a field
        //   + field exists
        //   + alias to a field exists
        //   + error

        if let Some(no_dot_name) = name.strip_prefix('.') {
            if self.tmp_vars.remove(no_dot_name).is_some() {
                bunt::println!("Deleted temporary variable `{[cyan+dimmed]}`.", name)
            } else {
                // check aliases
                for (tmp_var, fdata) in self.tmp_vars.iter_mut() {
                    if fdata.aliases.remove(no_dot_name) {
                        bunt::println!(
                            "Deleted alias `{[cyan+dimmed]}` of temporary variable `{[cyan]}`.",
                            name,
                            tmp_var,
                        );
                        return Ok(());
                    }
                }
                // Not found, so error.
                bunt::println!(
                    "Could not delete `{[cyan+dimmed]}` because it is neither an existing \
                    temporary variable nor an alias to one.",
                    name,
                );
            }
        } else if self.fields.remove(name).is_some() {
            bunt::println!("Deleted field `{[cyan+dimmed]}`.", name);
        } else {
            // check aliases
            for (field, fdata) in self.fields.iter_mut() {
                if fdata.aliases.remove(name) {
                    bunt::println!(
                        "Deleted alias `{[cyan+dimmed]}` of field `{[cyan]}`.",
                        name,
                        field,
                    );
                    return Ok(());
                }
            }
            // Not found, so error.
            bunt::println!(
                "Could not delete `{[cyan+dimmed]}` because it is neither an existing \
                field nor an alias to one.",
                name,
            );
        }
        Ok(())
    }
}
