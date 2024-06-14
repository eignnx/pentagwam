use owo_colors::OwoColorize;

use crate::human_powered_vm::script::Script;
use crate::human_powered_vm::styles::{self, bad_instr, bad_name, err_tok, name, note};
use crate::human_powered_vm::{error::Error, error::Result, HumanPoweredVm};
use crate::vals::{lval::LVal, rval::RVal, slice::Region, val::Val};

impl HumanPoweredVm {
    pub(super) fn print_rval(&self, rval: &RVal) -> Result<()> {
        let val = self.eval_to_val(rval)?;
        if let Val::Slice { region, start, len } = val {
            self.print_slice(region, start, len)?;
        } else {
            println!("=> {}", self.mem.display(&val).style(styles::val()));
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
                    println!(
                        "{:04}: {}",
                        i.style(note()),
                        self.mem.display(cell).style(styles::cell())
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
                    println!(
                        "{:04}: {}",
                        i.style(note()),
                        self.mem.display(instr).style(styles::instr())
                    );
                }
                println!("{:-^20}", "");
            }
        }
        Ok(())
    }

    pub(super) fn assign_to_lval(&mut self, lval_name: &str, rhs_name: &str) -> Result<()> {
        use chumsky::prelude::*;
        let lval = LVal::parser().then_ignore(end()).parse(lval_name)?;
        let rval = RVal::parser().then_ignore(end()).parse(rhs_name)?;
        let _val = self.lval_set(&lval, &rval)?;
        Ok(())
    }

    pub(super) fn add_alias(&mut self, new_name: &str, old_name: &str) -> Result<()> {
        // First check if the old name is for a temporary variable.
        if let Some(no_dot_old_name) = old_name.strip_prefix('.') {
            // Ensure the new name also looks like a temporary variable.
            let Some(no_dot_new_name) = new_name.strip_prefix('.') else {
                println!(
                    "{} An alias to a temporary variable must begin with a dot.",
                    err_tok()
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
                println!(
                    "{} Cannot create alias `{new_name}` of temporary \
                    variable `{old_name}` because `{new_name}` already aliases \
                    temporary variable `{existing_name}`.",
                    err_tok(),
                    old_name = old_name.style(name()),
                    new_name = new_name.style(bad_name()),
                    existing_name = existing_name.style(name()),
                );
                return Ok(());
            }

            // Add alias.
            if let Some(fdata) = self.tmp_vars.get_mut(no_dot_old_name) {
                fdata.aliases.insert(no_dot_new_name.to_string());
                println!(
                    "Aliased temporary variable `{old_name}` as \
                    `{new_name}`.",
                    old_name = old_name.style(name()),
                    new_name = new_name.style(name()),
                );
            } else if let Some((tmp_var_name, fdata)) = self
                .tmp_vars
                .iter_mut()
                .find(|(_, fdata)| fdata.aliases.contains(no_dot_old_name))
            {
                // Actually add the alias under `field_name` because `old_name` aliases it.
                fdata.aliases.insert(no_dot_new_name.to_string());
                let tmp_var_name = format!(".{tmp_var_name}");
                println!(
                    "Aliased `{old_name}` (temporary variable `{tmp_var_name}`) as `{new_name}`.",
                    old_name = old_name.style(name()),
                    tmp_var_name = tmp_var_name.style(name()),
                    new_name = new_name.style(name()),
                );
            } else {
                println!(
                    "{} Can't alias `{old_name}` as `{new_name}` because temporary variable `{old_name}` doesn't exist.",
                    err_tok(),
                    old_name = old_name.style(bad_name()),
                    new_name = new_name.style(bad_name()),
                );
            }
        } else {
            // Check that the alias doesn't already exist.
            if let Some(existing_name) = self
                .fields
                .iter()
                .find_map(|(name, fdata)| fdata.aliases.contains(new_name).then_some(name))
            {
                println!(
                    "{} Cannot create alias `{new_name_bad}` of field `{old_name}` \
                    because `{new_name}` already aliases field `{existing_name}`.",
                    err_tok(),
                    old_name = old_name.style(name()),
                    new_name_bad = new_name.style(bad_name()),
                    new_name = new_name.style(name()),
                    existing_name = existing_name.style(name()),
                );
                return Ok(());
            }

            if let Some(fdata) = self.fields.get_mut(old_name) {
                // Check that `new_name` doesn't begin with a dot.
                if new_name.starts_with('.') {
                    println!(
                        "{} An alias of a field cannot begin with a dot (`.`).",
                        err_tok()
                    );
                    return Ok(());
                }
                fdata.aliases.insert(new_name.to_string());
                println!(
                    "Aliased field `{old_name}` as `{new_name}`.",
                    old_name = old_name.style(name()),
                    new_name = new_name.style(name()),
                );
            } else if let Some((field_name, fdata)) = self
                .fields
                .iter_mut()
                .find(|(_, fdata)| fdata.aliases.contains(old_name))
            {
                // Actually add the alias under `field_name` because `old_name` aliases it.
                if new_name.starts_with('.') {
                    println!(
                        "{} An alias of a field cannot begin with a dot (`.`).",
                        err_tok()
                    );
                    return Ok(());
                }
                fdata.aliases.insert(new_name.to_string());
                println!(
                    "Aliased `{old_name}` (field `{field_name}`) as `{new_name}`.",
                    old_name = old_name.style(name()),
                    field_name = field_name.style(name()),
                    new_name = new_name.style(name()),
                );
            } else {
                println!(
                    "{} Can't alias `{old_name}` as `{new_name}` because field \
                    `{old_name}` doesn't exist.",
                    err_tok(),
                    old_name = old_name.style(bad_name()),
                    new_name = new_name.style(bad_name()),
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
                println!("Deleted temporary variable `{}`.", name.style(bad_name()))
            } else {
                // check aliases
                for (tmp_var, fdata) in self.tmp_vars.iter_mut() {
                    if fdata.aliases.remove(no_dot_name) {
                        println!(
                            "Deleted alias `{}` of temporary variable `{}`.",
                            name.style(bad_name()),
                            tmp_var.style(styles::name()),
                        );
                        return Ok(());
                    }
                }
                // Not found, so error.
                println!(
                    "Could not delete `{}` because it is neither an existing \
                    temporary variable nor an alias to one.",
                    name.style(bad_name()),
                );
            }
        } else if self.fields.remove(name).is_some() {
            println!("Deleted field `{}`.", name.style(bad_name()));
        } else {
            // check aliases
            for (field, fdata) in self.fields.iter_mut() {
                if fdata.aliases.remove(name) {
                    println!(
                        "Deleted alias `{}` of field `{}`.",
                        name.style(bad_name()),
                        field.style(styles::name()),
                    );
                    return Ok(());
                }
            }
            // Not found, so error.
            println!(
                "Could not delete `{}` because it is neither an existing field \
                nor an alias to one.",
                name.style(bad_name()),
            );
        }
        Ok(())
    }

    pub(super) fn edit_script(&mut self, rest: &[&str]) -> Result<()> {
        let (instr_name, doc_comment) = match rest {
            [] => {
                if let Some(instr) = self.program.get(self.instr_ptr()) {
                    (instr.instr_name().to_string(), instr.doc_comment())
                } else {
                    println!(
                        "{}",
                        "No current instruction to which to associated a script.".style(note())
                    );
                    return Ok(());
                }
            }
            [instr_name] => {
                // Check that it's a valid instruction name.
                if let Some(instr) = self
                    .program
                    .iter()
                    .find(|instr| instr.instr_name() == *instr_name)
                {
                    (instr_name.to_string(), instr.doc_comment())
                } else {
                    println!(
                        "{} The name `{}` is not a valid instruction name.",
                        err_tok(),
                        instr_name.style(bad_instr())
                    );
                    return Ok(());
                }
            }
            other => {
                println!(
                    "{} `script` command expects 0 or 1 arguments, got {}.",
                    err_tok(),
                    other.len()
                );
                return Ok(());
            }
        };

        let script = self
            .instr_scripts
            .entry(instr_name)
            .or_insert_with_key(|instr_name| {
                let mut default_text = String::new();
                default_text += &doc_comment
                    .unwrap_or_default()
                    .lines()
                    .map(|line| format!("# {line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                default_text += &format!("\n\n# Script for instruction `{instr_name}`\n");
                default_text += "# Feel free to edit this file however you like.\n";
                default_text +=
                    "# Remember to use `$1`, `$2`, etc to refer to the instruction's parameters.\n";
                Script::parse(&default_text).unwrap()
            });

        println!("{}", "Opening associated script in editor...".dimmed());
        println!();
        if let Some(preferred_editor) = &self.preferred_editor {
            std::env::set_var("EDITOR", preferred_editor);
        }
        *script = Script::parse(&edit::edit(script.to_string())?)?;
        println!("```\n{script}\n```");

        Ok(())
    }
}
