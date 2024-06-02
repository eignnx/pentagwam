use derive_more::From;
use pentagwam::{
    bc::instr::Instr,
    defs::{CellRef, Sym},
    mem::{DisplayViaMem, Mem},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    io::{Read, Write},
    ops::ControlFlow,
};

use crate::human_powered_vm::{
    error::{Error, Result},
    vals::{lval::LVal, rval::RVal, val::Val, valty::ValTy},
};

pub mod cmds;
pub mod error;
pub mod eval;
pub mod scenario;
pub mod vals;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HumanPoweredVm {
    fields: BTreeMap<String, FieldData>,
    #[serde(skip)]
    tmp_vars: BTreeMap<String, FieldData>,
    instr_ptr: usize,
    #[serde(skip)]
    mem: Mem,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldData {
    #[serde(skip)]
    value: Val,
    ty: ValTy,
    aliases: BTreeSet<String>,
}

impl FieldData {
    fn assign_val(&mut self, rhs: Val) -> Result<()> {
        if self.ty != rhs.ty() {
            return Err(Error::AssignmentTypeError {
                expected: self.ty.to_string(),
                received: rhs.ty(),
            });
        }
        self.value = rhs;
        Ok(())
    }
}

const SAVE_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/SAVE.ron");

impl Drop for HumanPoweredVm {
    fn drop(&mut self) {
        let self_ron = ron::ser::to_string_pretty(
            &self,
            ron::ser::PrettyConfig::default()
                .struct_names(true)
                .depth_limit(3),
        )
        .unwrap();
        let mut file = std::fs::File::create(SAVE_FILE).unwrap_or_else(|e| {
            println!("Could not open save file `{SAVE_FILE}` due to error: {e}");
            println!("DUMP SAVE DATA:");
            println!("---------------");
            println!("{self_ron}");
            println!("---------------");
            std::process::exit(2);
        });
        write!(file, "{}", self_ron).unwrap();
    }
}

impl HumanPoweredVm {
    pub fn new() -> Result<Self> {
        match std::fs::File::open(SAVE_FILE) {
            Ok(mut file) => {
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                let mut vm: Self = ron::from_str(&buf)?;
                vm.populate_default_field_values();
                Ok(vm)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!(
                    "No SAVE.ron save file found. On exit, one will be \
                     created at: {SAVE_FILE}"
                );
                Ok(Default::default())
            }
            Err(e) => Err(e.into()),
        }
    }

    fn populate_default_field_values(&mut self) {
        // We'd like for the Deserialize implementation to look at the `ValTy`
        // of the field and generate a default based on that, but I don't know
        // how to do that. So we'll just post-process a bit.
        for (_field, data) in self.fields.iter_mut() {
            data.value = data.ty.default_val();
        }
    }

    fn prompt(&self, prompt: &str) -> String {
        print!("({prompt})=> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        println!();
        input.trim().to_string()
    }

    pub fn run<L: Display, S: DisplayViaMem>(&mut self, program: &[Instr<L, S>]) -> Result<()> {
        loop {
            println!();
            if let Some(instr) = program.get(self.instr_ptr) {
                println!("instr #{:04}: {}", self.instr_ptr, self.mem.display(instr));
                println!();
            } else {
                println!("[Instruction pointer beyond end of program]");
                println!();
            }

            let cmd = self.prompt("Enter a command");
            match self.handle_cmd(&cmd, program) {
                Ok(ControlFlow::Break(())) => break,
                Ok(ControlFlow::Continue(())) => continue,
                Err(e) => println!("!> {e}"),
            }
        }
        Ok(())
    }

    fn handle_cmd<L: std::fmt::Display, S: DisplayViaMem>(
        &mut self,
        cmd: &str,
        program: &[Instr<L, S>],
    ) -> Result<ControlFlow<()>> {
        let cmd_split = cmd.split_whitespace().collect::<Vec<_>>();
        match &cmd_split[..] {
            [] => {
                println!("=> No command entered.");
                println!();
                self.print_help()
            }
            ["help" | "h" | "?" | "--help"] => self.print_help(),
            ["docs" | "doc" | "d"] => {
                // Print out the doc-comment associated with the current instruction.
                if let Some(instr) = program.get(self.instr_ptr) {
                    if let Some(docs) = instr.doc_comment() {
                        println!("{:-^80}", "INSTRUCTION DOCUMENTATION");
                        println!();
                        println!("{:^80}", self.mem.display(instr).to_string());
                        println!();
                        println!("{docs}");
                        println!("{:-<80}", "");
                    } else {
                        println!(
                            "!> No documentation available for instruction `{}`",
                            self.mem.display(instr)
                        );
                    }
                }
            }
            ["quit" | "q" | ":wq" | ":q"] => {
                println!("Saving field declarations and exiting...");
                return Ok(ControlFlow::Break(()));
            }
            ["fields" | "f"] => {
                println!("Virtual Machine Fields:");
                for (field, fdata) in self.fields.iter() {
                    let def = format!(
                        "\t{field}: {} = {}",
                        fdata.ty,
                        self.mem.display(&fdata.value)
                    );
                    if !fdata.aliases.is_empty() {
                        let joined = fdata
                            .aliases
                            .iter()
                            .map(AsRef::as_ref)
                            .collect::<Vec<&str>>()
                            .join(", ");
                        let aliases = format!("aliases: {joined}");
                        println!("{def:<40}{aliases};");
                    } else {
                        println!("{def};");
                    }
                }

                println!();
                println!("Temporary Variables:");
                if self.tmp_vars.is_empty() {
                    println!("\t<no temporary variables defined>");
                } else {
                    for (var_name, fdata) in self.tmp_vars.iter() {
                        print!(
                            "\t.{var_name}: {} = {}",
                            fdata.ty,
                            self.mem.display(&fdata.value)
                        );
                        if !fdata.aliases.is_empty() {
                            print!("\t\taliases: ");
                            for (i, alias) in fdata.aliases.iter().enumerate() {
                                print!("{sep}.{alias}", sep = if i > 0 { ", " } else { "" });
                            }
                        }
                        println!(";");
                    }
                }
            }
            ["list" | "l", rest @ ..] => {
                println!("Program Listing:");
                self.program_listing(rest, program)?;
            }
            ["next" | "n"] => {
                self.instr_ptr += 1;
                println!("Advanced to next instruction.");
            }
            ["del", field_name] => {
                if self.fields.remove(*field_name).is_some() {
                    println!("Deleted field `{field_name}`.");
                } else {
                    println!("Field `{field_name}` can't be deleted because it doesn't exist.")
                }
            }
            ["push", "term" | "tm", rest @ ..] => {
                use chumsky::Parser;
                let term_text: String = rest.join(" ");
                let term_parser = pentagwam::syntax::Term::parser();
                let term = term_parser.parse::<_, &str>(term_text.as_str())?;
                let cell_ref = term.serialize(&mut self.mem);
                println!("Serialized Prolog term `{term_text}` into memory at `{cell_ref}`.");
            }
            ["push", rval] => {
                let rval: RVal = rval.parse()?;
                let val = self.eval_to_val(&rval)?;
                let cell = val.try_as_cell()?;
                self.mem.push(cell);
                println!("Pushed `{}` onto top of heap.", self.mem.display(&val));
            }
            [_, "=", tm @ ("term" | "tm"), ..] => {
                println!("!> Use `<lval> {tm} <- <rval>` to assign a value to an l-value.");
            }
            [_, "=", ..] => {
                println!("!> Use `<lval> <- <rval>` to assign a value to an l-value.");
            }
            [lval, "<-", "term" | "tm", rest @ ..] => {
                use chumsky::Parser;
                let term_text: String = rest.join(" ");
                let term_parser = pentagwam::syntax::Term::parser();
                let term = term_parser.parse::<_, &str>(term_text.as_str())?;
                let cell_ref = term.serialize(&mut self.mem);
                println!("Serialized Prolog term `{term_text}` into memory at `{cell_ref}`.");
                let lval: LVal = lval.parse()?;
                let rval: RVal = cell_ref.into();
                self.lval_set(&lval, &rval)?;
                println!(
                    "CellRef `{cell_ref}` saved into `{}`.",
                    self.mem.display(&lval)
                );
            }
            [lval, "<-", rhs] => {
                self.assign_to_lval(lval, rhs)?;
            }
            ["alias", new_name, "->", old_name] => {
                if let Some(old_name) = old_name.strip_prefix('.') {
                    let Some(new_name) = new_name.strip_prefix('.') else {
                        println!("!> An alias to a temporary variable must begin with a dot.");
                        return Ok(ControlFlow::Continue(()));
                    };

                    if let Some(fdata) = self.tmp_vars.get_mut(old_name) {
                        fdata.aliases.insert(new_name.to_string());
                        println!("Aliased `.{old_name}` as `.{new_name}`.");
                    } else {
                        println!(
                            "!> Can't alias `.{old_name}` as `.{new_name}` because \
                                  temporary variable `.{old_name}` doesn't exist."
                        );
                    }
                } else if let Some(fdata) = self.fields.get_mut(*old_name) {
                    fdata.aliases.insert(new_name.to_string());
                    println!("Aliased `{old_name}` as `{new_name}`.");
                } else {
                    println!("!> Can't alias `{old_name}` as `{new_name}` because `{old_name}` doesn't exist.");
                }
            }
            ["unalias", alias, "->", field] => {
                if let Some(fdata) = self.fields.get_mut(*field) {
                    if fdata.aliases.remove(*alias) {
                        println!("Unaliased `{alias}` from `{field}`.");
                    } else {
                        println!("!> Can't unalias `{alias}` from `{field}` because `{alias}` isn't an alias of `{field}`.");
                    }
                } else {
                    println!("!> Can't unalias `{alias}` from `{field}` because field `{field}` doesn't exist.");
                }
            }
            [tm @ ("term" | "tm"), rest @ ..] => {
                // Display a Prolog term
                use chumsky::Parser;
                let term_text: String = rest.join(" ");
                let term_parser = pentagwam::syntax::Term::parser();
                let term = term_parser.parse::<_, &str>(term_text.as_str())?;
                println!("=> {tm} {term}");
            }
            rval => {
                self.print_rval(&rval.join(" ").to_string())?;
            }
        }
        Ok(ControlFlow::Continue(()))
    }

    fn print_help(&self) {
        println!("{:-^80}", "COMMAND DOCUMENTATION");
        println!();
        println!("Commands:");
        println!("  <lval> <- <rval> - Assign the value of <rval> to <lval>.");
        println!("  <lval> <- tm <tm>");
        println!("                   - Assign the Prolog term <tm> to <lval>.");
        println!("  <rval>           - Print the value of <rval>.");
        println!("  tm <rval>        - Print the Prolog term residing in memory");
        println!("                     at CellRef <rval>.");
        println!("  alias <new> -> <old>");
        println!("                   - Alias <old> as <new>.");
        println!("  unalias <alias> -> <field>");
        println!("                   - Unalias <alias> from <field>.");
        println!("  del <field>      - Delete the field <field>.");
        println!("  push <rval>      - Push the value of <rval> onto the heap.");
        println!("  fields | f       - Print all the data fields of the VM.");
        println!("  list | l [from|next|prev|last|first <n>]");
        println!("                   - Print a program listing.");
        println!("  docs | doc | d   - Print the documentation for the current");
        println!("                     instruction.");
        println!("  next | n         - Advance to the next instruction.");
        println!("  quit | q         - Quit the program, saving any field declarations.");
        println!("  help | h | ?     - Print this help message.");
        println!();
        println!("  Expression Language:");
        println!();
        println!("  L-Values: values which represent a memory location which");
        println!("              can be assigned to.");
        println!("    <lval> ::= <field> | <tmp_var> | instr_ptr | <rval>.*");
        println!("             | <rval>[<rval>]");
        println!();
        println!("  R-Values: expressions which can evaluate to a base value (<val>).");
        println!("    <rval> ::= <usize> | <i32> | <sym> | <tmp_var> | <field>");
        println!("             | <rva>.& | <rval>.* | <rval>[<rval>] ");
        println!("             | <cell_ref> | <cell>");
        println!();
        println!("    <val>   ::= <usize> | <i32> | <sym> | <cell_ref> | <cell>");
        println!("    <usize> ::= 0 | 1 | 2 | ...");
        println!("    <i32>   ::= +0 | -0 | +1 | -1 | +2 | -2 | ...");
        println!("    <cell>  ::= Int(<i32>) | Sym(<sym>) | Ref(<cell_ref>)");
        println!("              | Rcd(<cell_ref>) | Sig(<functor>)");
        println!("              | Lst(<cell_ref>) | Nil");
        println!();
        println!("    <functor>  ::= example1/0 | my_functor/3 | '*'/2 | ...");
        println!("    <cell_ref> ::= @<usize>");
        println!("    <field>    ::= example1 | ExAmPlE2 | ...");
        println!("    <tmp_var>  ::= .example1 | .ExAmPlE2 | ...");
        println!("    <sym> ::= :example1 | :ExAmPlE2 | :'example with spaces'");
        println!("            | :'123' | ...");
        println!();
        println!("{:-<80}", "");
    }

    pub fn intern_sym(&self, text: &str) -> Sym {
        self.mem.intern_sym(text)
    }
}
