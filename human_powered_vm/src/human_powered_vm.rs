use chumsky::{primitive::end, Parser};
use pentagwam::{
    cell::Functor,
    defs::Sym,
    mem::{DisplayViaMem, Mem},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    io::{Read, Write},
    ops::ControlFlow,
};

use crate::human_powered_vm::error::{Error, Result};
use crate::vals::{
    lval::LVal,
    rval::RVal,
    slice::{Idx, Len, Slice},
    val::Val,
    valty::ValTy,
};

pub mod builtin_fields;
pub mod cmds;
pub mod error;
pub mod eval;
pub mod help;
pub mod scenario;

pub type Instr = pentagwam::bc::instr::Instr<Functor<String>, String>;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HumanPoweredVm {
    pub fields: BTreeMap<String, FieldData>,
    pub instr_scripts: BTreeMap<String, String>,
    #[serde(skip)]
    pub tmp_vars: BTreeMap<String, FieldData>,
    #[serde(skip)]
    pub mem: Mem,
    #[serde(skip)]
    pub program: Vec<Instr>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldData {
    #[serde(skip)]
    pub value: Val,
    pub ty: ValTy,
    pub default: Option<Val>,
    pub aliases: BTreeSet<String>,
}

impl FieldData {
    fn assign_val(&mut self, rhs: Val, mem: &Mem) -> Result<()> {
        self.value = rhs
            .try_convert(self.ty, mem)
            .map_err(|_| Error::AssignmentTypeError {
                expected: self.ty.to_string(),
                received: rhs.ty(),
            })?;
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
            bunt::println!(
                "{$red}Could not open save file `{}` due to error: {}{/$}",
                SAVE_FILE,
                e
            );
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
            data.value = data
                .default
                .clone()
                .unwrap_or_else(|| data.ty.default_val(&self.mem));
        }

        self.setup_default_fields();
    }

    pub fn load_program(&mut self, program: Vec<Instr>) -> &mut Self {
        self.program = program;
        self
    }

    fn prompt(&self, prompt: &str) -> String {
        print!("({prompt}): ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        println!();
        input.trim().to_string()
    }

    pub fn run<L: Display, S: DisplayViaMem>(&mut self) -> Result<()> {
        loop {
            self.update_builtin_fields();
            println!();
            if let Some(instr) = self.program.get(self.instr_ptr()) {
                bunt::println!(
                    "{$dimmed}instr #{:04}:{/$} {[bold+italic]}",
                    self.instr_ptr(),
                    self.mem.display(instr)
                );
            } else {
                bunt::println!(
                    "{$dimmed}instr #{:04}:{/$} {$dimmed+italic}[instr pointer beyond end of program]{/$}",
                    self.instr_ptr(),
                );
            }

            let cmd = self.prompt("Enter a command");
            match self.handle_cmd(&cmd) {
                Ok(ControlFlow::Break(())) => break,
                Ok(ControlFlow::Continue(())) => continue,
                Err(e) => bunt::println!("{$red}!>{/$} {}", e),
            }
        }
        Ok(())
    }

    fn handle_cmd(&mut self, cmd: &str) -> Result<ControlFlow<()>> {
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
                if let Some(instr) = self.program.get(self.instr_ptr()) {
                    if let Some(docs) = instr.doc_comment() {
                        println!("{:-^80}", "INSTRUCTION DOCUMENTATION");
                        println!();
                        bunt::println!(
                            "{[bold+intense+italic]:^80}",
                            self.mem.display(instr).to_string()
                        );
                        println!();
                        println!("{docs}");
                        println!("{:-<80}", "");
                    } else {
                        bunt::println!(
                            "{$red}!>{/$} No documentation available for instruction `{[italic+intense+bold]}`",
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
                    bunt::print!(
                        "\t{[cyan]}: {[green]} = {[yellow]}",
                        field,
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
                        bunt::println!("\t\t\t\t{[italic]};", aliases);
                    } else {
                        println!();
                    }
                }

                println!();
                println!("Temporary Variables:");
                if self.tmp_vars.is_empty() {
                    bunt::println!("\t{$italic+dimmed}No temporary variables defined.{/$}");
                } else {
                    for (var_name, fdata) in self.tmp_vars.iter() {
                        bunt::print!(
                            "\t.{[cyan]}: {[green]} = {[yellow]}",
                            var_name,
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
            ["script" | "s", rest @ ..] => {
                self.edit_script(rest)?;
            }
            ["del", "script" | "s", instr_name] => {
                if let Some(script) = self.instr_scripts.remove(*instr_name) {
                    bunt::println!(
                        "{[dimmed]}",
                        format!("Deleted script for instruction `{instr_name}`."),
                    );
                    println!("```\n{script}\n```");
                } else {
                    bunt::println!(
                        "{$red}!>{/$} Could not find an existing script for `{}`.",
                        instr_name
                    );
                }
            }
            ["list" | "l", rest @ ..] => {
                let text = rest.join("");
                let rval = text.parse()?;
                let sliced = RVal::IndexSlice(
                    Box::new(rval),
                    Box::new(Slice {
                        idx: Idx::Lo,
                        len: Len::PosInf,
                    }),
                );
                self.print_rval(&sliced)?;
            }
            ["next" | "n"] => {
                *self.instr_ptr_mut() += 1;
                bunt::println!("{$dimmed}Advanced to next instruction.{/$}");
            }
            ["del", name] => {
                self.delete_name(name)?;
            }
            ["push", "term" | "tm", rest @ ..] => {
                let term_text: String = rest.join(" ");
                let term_parser = pentagwam::syntax::Term::parser();
                let term = term_parser.parse::<_, &str>(term_text.as_str())?;
                let cell_ref = term.serialize(&mut self.mem);
                bunt::println!(
                    "Serialized Prolog term `{[yellow]}` into memory at `{[yellow]}`.",
                    term_text,
                    cell_ref
                );
            }
            ["push", rval] => {
                let rval: RVal = rval.parse()?;
                let val = self.eval_to_val(&rval)?;
                let cell = val.try_as_cell(&self.mem)?;
                self.mem.push(cell);
                bunt::println!(
                    "Pushed `{[yellow]}` onto top of heap.",
                    self.mem.display(&val)
                );
            }
            [_, "=", tm @ ("term" | "tm"), ..] => {
                bunt::println!("{$red}!>{/$} Use `<lval> {tm} {$bold+intense+red}<-{/$} <rval>` to assign to an l-value.", tm = tm);
            }
            [_, "=", ..] => {
                bunt::println!(
                    "{$red}!>{/$} Use `<lval> {$bold+intense+red}<-{/$} <rval>` to assign to an l-value."
                );
            }
            [lval, "<-", "term" | "tm", rest @ ..] => {
                let term_text: String = rest.join(" ");
                let term_parser = pentagwam::syntax::Term::parser();
                let term = term_parser.parse::<_, &str>(term_text.as_str())?;
                let cell_ref = term.serialize(&mut self.mem);
                bunt::println!(
                    "Serialized Prolog term `{[yellow]term_text}` into memory at `{[yellow]cell_ref}`.",
                    term_text = term_text,
                    cell_ref = cell_ref
                );
                let lval: LVal = lval.parse()?;
                let rval: RVal = cell_ref.into();
                self.lval_set(&lval, &rval)?;
                bunt::println!(
                    "CellRef `{[yellow]cell_ref}` saved into `{[yellow]}`.",
                    self.mem.display(&lval),
                    cell_ref = cell_ref,
                );
            }
            [lval, "<-", rhs] => {
                self.assign_to_lval(lval, rhs)?;
            }
            ["alias", new_name, "->", old_name] => {
                self.add_alias(new_name, old_name)?;
            }
            [tm @ ("term" | "tm"), rest @ ..] => {
                // Display a Prolog term
                let term_text: String = rest.join(" ");
                let term_parser = pentagwam::syntax::Term::parser();
                let term = term_parser.parse::<_, &str>(term_text.as_str())?;
                bunt::println!("=> {tm} {[yellow]term}", tm = tm, term = term);
            }
            rval => {
                let rval = RVal::parser().then_ignore(end()).parse(rval.join(" "))?;
                self.print_rval(&rval)?;
            }
        }
        Ok(ControlFlow::Continue(()))
    }

    pub fn intern_sym(&self, text: &str) -> Sym {
        self.mem.intern_sym(text)
    }
}
