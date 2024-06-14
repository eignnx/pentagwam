use chumsky::{primitive::end, Parser};
use owo_colors::OwoColorize;
use pentagwam::{
    cell::Functor,
    defs::Sym,
    mem::{DisplayViaMem, Mem},
};
use script::Script;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    io::{Read, Write},
    ops::ControlFlow,
};
use styles::{err_tok, instr, name, note, val, valty};

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
pub mod script;
pub mod styles;

pub type Instr = pentagwam::bc::instr::Instr<Functor<String>, String>;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HumanPoweredVm {
    pub fields: BTreeMap<String, FieldData>,
    pub instr_scripts: BTreeMap<String, Script>,
    #[serde(skip)]
    pub tmp_vars: BTreeMap<String, FieldData>,
    #[serde(skip)]
    pub mem: Mem,
    #[serde(skip)]
    pub program: Vec<Instr>,
    pub preferred_editor: Option<String>,
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
                .depth_limit(4),
        )
        .unwrap();
        let mut file = std::fs::File::create(SAVE_FILE).unwrap_or_else(|e| {
            println!(
                "{} Could not open save file `{SAVE_FILE}` due to error: {e}",
                err_tok()
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
        print!("({}): ", prompt.style(note()));
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
                println!(
                    "{} {}",
                    format!("instr #{:04}:", self.instr_ptr()).style(note()),
                    self.mem.display(instr)
                );
            } else {
                println!(
                    "{}",
                    format!(
                        "instr #{:04}: [instr pointer beyond end of program]",
                        self.instr_ptr(),
                    )
                    .style(note()),
                );
            }

            let cmd = self.prompt("Enter a command");
            match self.handle_cmd(&cmd) {
                Ok(ControlFlow::Break(())) => break,
                Ok(ControlFlow::Continue(())) => continue,
                Err(e) => println!("{} {e}", err_tok()),
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
                        println!(
                            "{:^80}",
                            self.mem.display(instr).to_string().style(styles::instr())
                        );
                        println!();
                        println!("{docs}");
                        println!("{:-<80}", "");
                    } else {
                        println!(
                            "{} No documentation available for instruction `{}`",
                            err_tok(),
                            self.mem.display(instr).style(styles::instr())
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
                    let decl = format!(
                        "{}: {} = {}",
                        field.style(name()),
                        fdata.ty.style(valty()),
                        self.mem.display(&fdata.value).style(val())
                    );
                    if !fdata.aliases.is_empty() {
                        let joined = fdata
                            .aliases
                            .iter()
                            .map(AsRef::as_ref)
                            .collect::<Vec<&str>>()
                            .join(", ");
                        let aliases = format!("aliases: {joined};");
                        println!("\t{decl:<40}{:>40}", aliases.style(note()));
                    } else {
                        println!("\t{decl};");
                    }
                }

                println!();
                println!("Temporary Variables:");
                if self.tmp_vars.is_empty() {
                    println!("\t{}", "No temporary variables defined.".style(note()));
                } else {
                    for (var_name, fdata) in self.tmp_vars.iter() {
                        print!(
                            "\t.{}: {} = {}",
                            var_name.style(name()),
                            fdata.ty.style(valty()),
                            self.mem.display(&fdata.value).style(val())
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
            ["config", "editor"] => {
                println!(
                    "Choose a preferred text editor for editing instruction-associated scripts.\
                    Current preferred editor is `{}`.",
                    self.preferred_editor.as_deref().unwrap_or("<none>")
                );
                let mut choices = vec![];
                for (category, editors) in script::EDITORS_AVAILABLE {
                    println!("  {category}:");
                    for editor in *editors {
                        println!("    {idx}. {editor}", idx = choices.len() + 1);
                        choices.push(editor);
                    }
                }
                loop {
                    let input = self.prompt(&format!(
                        "Enter a number in the range 1..={}",
                        choices.len()
                    ));

                    if ["none", "<none>", "0", ""].contains(&input.to_ascii_lowercase().as_str()) {
                        self.preferred_editor = None;
                        println!("Resetting to default text editor.");
                        break;
                    } else if let Ok(n) = input.parse::<usize>() {
                        if (1..=choices.len()).contains(&n) {
                            let choice = choices[n - 1];
                            self.preferred_editor = Some(choice.to_string());
                            println!("Preferred editor set to `{choice}`.");
                            break;
                        } else {
                            println!("{} Choice out of valid range.", err_tok());
                        }
                    } else {
                        println!("{} Please enter a positive integer.", err_tok());
                    }
                }
            }
            ["script" | "s", rest @ ..] => {
                self.edit_script(rest)?;
            }
            ["run" | "r", "script" | "s"] | ["rs"] => {
                if let Some(instr) = self.program.get(self.instr_ptr()).cloned() {
                    if let Some(script) = self.instr_scripts.get(instr.instr_name()) {
                        println!(
                            "Running script for `{}` instruction...",
                            instr.instr_name().style(styles::instr())
                        );
                        let script = script.clone();
                        script.exec(self)?;
                    } else {
                        println!(
                            "{} No script found for instruction `{}`.",
                            err_tok(),
                            instr.instr_name().style(styles::bad_instr())
                        );
                    }
                } else {
                    println!(
                        "{} No instruction found at program index `{}`.",
                        err_tok(),
                        self.instr_ptr()
                    );
                }
            }
            ["del", "script" | "s", instr_name] => {
                if let Some(script) = self.instr_scripts.remove(*instr_name) {
                    println!(
                        "{}",
                        format!("Deleted script for instruction `{instr_name}`.").style(note()),
                    );
                    println!("```\n{script}\n```");
                } else {
                    println!(
                        "{} Could not find an existing script for `{}`.",
                        err_tok(),
                        instr_name.style(instr())
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
                println!("{}", "Advanced to next instruction.".style(note()));
            }
            ["del", name] => {
                self.delete_name(name)?;
            }
            ["push", "term" | "tm", rest @ ..] => {
                let term_text: String = rest.join(" ");
                let term_parser = pentagwam::syntax::Term::parser();
                let term = term_parser.parse::<_, &str>(term_text.as_str())?;
                let cell_ref = term.serialize(&mut self.mem);
                println!(
                    "Serialized Prolog term `{}` into memory at `{}`.",
                    term_text.style(val()),
                    cell_ref.style(val())
                );
            }
            ["push", rval] => {
                let rval: RVal = rval.parse()?;
                let val = self.eval_to_val(&rval)?;
                let cell = val.try_as_cell(&self.mem)?;
                self.mem.push(cell);
                println!(
                    "Pushed `{}` onto top of heap.",
                    self.mem.display(&val).style(styles::val())
                );
            }
            [_, "=", tm @ ("term" | "tm"), ..] => {
                println!(
                    "{} Use `<lval> {tm} {arr} <rval>` to assign to an l-value.",
                    err_tok(),
                    arr = "<-".bright_red()
                );
            }
            [_, "=", ..] => {
                println!(
                    "{} Use `<lval> {arr} <rval>` to assign to an l-value.",
                    err_tok(),
                    arr = "<-".bright_red()
                );
            }
            [lval, "<-", "term" | "tm", rest @ ..] => {
                let term_text: String = rest.join(" ");
                let term_parser = pentagwam::syntax::Term::parser();
                let term = term_parser.parse::<_, &str>(term_text.as_str())?;
                let cell_ref = term.serialize(&mut self.mem);
                println!(
                    "Serialized Prolog term `{term_text}` into memory at `{cell_ref}`.",
                    term_text = term_text.style(val()),
                    cell_ref = cell_ref.style(val())
                );
                let lval: LVal = lval.parse()?;
                let rval: RVal = cell_ref.into();
                self.lval_set(&lval, &rval)?;
                println!(
                    "CellRef `{}` saved into `{}`.",
                    cell_ref.style(val()),
                    self.mem.display(&lval).style(val()),
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
                println!("=> {tm} {term}", tm = tm, term = term.style(val()));
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
