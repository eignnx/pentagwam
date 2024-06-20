use chumsky::{primitive::end, Parser};
use owo_colors::OwoColorize;
use pentagwam::{
    cell::Functor,
    defs::Sym,
    mem::{DisplayViaMem, Mem},
    syntax::Term,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    io::{Read, Write},
    ops::ControlFlow,
    path::PathBuf,
};
use styles::{err_tok, instr, note, val};

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
    #[serde(skip)]
    pub tmp_vars: BTreeMap<String, FieldData>,
    #[serde(skip)]
    pub mem: Mem,
    #[serde(skip)]
    pub program: Vec<Instr>,
    pub preferred_editor: Option<String>,
    #[serde(skip)]
    branch_stack: Vec<(Option<bool>, Cond)>,
    arenas: BTreeMap<String, Vec<Val>>,
}

#[derive(Debug)]
enum Cond {
    Consequent,
    Alternative,
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

const SAVE_DIR: &str = ".hpvm-save";
const FIELDS_FILE: &str = "fields.ron";
const SCRIPTS_DIR: &str = "scripts";

impl Drop for HumanPoweredVm {
    fn drop(&mut self) {
        let self_ron = ron::ser::to_string_pretty(
            &self,
            ron::ser::PrettyConfig::default()
                .struct_names(true)
                .depth_limit(4),
        )
        .expect("Serialization to RON failed!");

        self.drop_impl(&self_ron).unwrap_or_else(|e| {
            println!(
                "{} Could not save to `{FIELDS_FILE}` due to error: {e}",
                err_tok(),
            );
            println!("DUMP SAVE DATA:");
            println!("---------------");
            println!("{self_ron}");
            println!("---------------");
            std::process::exit(2);
        });
    }
}

impl HumanPoweredVm {
    pub fn save_dir_location() -> PathBuf {
        use std::env;
        PathBuf::from(env::var("HPVM_CWD").unwrap_or(env!("CARGO_MANIFEST_DIR").to_string()))
            .join(SAVE_DIR)
    }

    fn drop_impl(&mut self, self_ron: &str) -> Result<()> {
        use std::fs;
        fs::create_dir_all(Self::save_dir_location().join(SCRIPTS_DIR))?;
        let mut file = std::fs::File::create(Self::save_dir_location().join(FIELDS_FILE))?;
        write!(file, "{self_ron}")?;
        Ok(())
    }

    pub fn new() -> Result<Self> {
        let fields_file_location = Self::save_dir_location().join(FIELDS_FILE);
        match std::fs::File::open(&fields_file_location) {
            Ok(mut file) => {
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                let mut vm: Self = ron::from_str(&buf)?;
                vm.populate_default_field_values();
                Ok(vm)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!(
                    "No `{FIELDS_FILE}` save file found. On exit, one will be \
                     created at: {}",
                    fields_file_location.display()
                );
                Ok(Default::default())
            }
            Err(e) => Err(e.into()),
        }
    }

    fn populate_default_field_values(&mut self) {
        self.setup_builtin_fields();

        // We'd like for the Deserialize implementation to look at the `ValTy`
        // of the field and generate a default based on that, but I don't know
        // how to do that. So we'll just post-process a bit.
        for (_field, data) in self.fields.iter_mut() {
            data.value = data
                .default
                .clone()
                .unwrap_or_else(|| data.ty.default_val(&self.mem));
        }
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
                    self.mem.display(instr).style(styles::instr())
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

        match self.conditional_skip(&cmd_split)? {
            ControlFlow::Break(SkipReason::CmdSkipped) => {
                println!("=> {}", "Skipping command.".style(note()));
                return Ok(ControlFlow::Continue(()));
            }
            ControlFlow::Break(_) => return Ok(ControlFlow::Continue(())),
            ControlFlow::Continue(()) => {}
        }

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
            ["fields" | "f"] => self.print_fields()?,
            ["config", "editor"] => self.config_editor()?,
            ["script" | "s", rest @ ..] => {
                self.edit_script(rest)?;
            }
            ["run" | "r", "script" | "s"] | ["rs"] => {
                self.run_script()?;
            }
            ["del", "script" | "s", instr_name] => {
                if let Ok(instr_name) = instr_name.parse() {
                    if let Ok(script) = self.delete_script_file(instr_name) {
                        println!(
                            "{}",
                            format!("Deleted script for instruction `{instr_name}`.").style(note()),
                        );
                        println!("---\n{script}\n---");
                    } else {
                        println!(
                            "{} Could not find an existing script for `{}`.",
                            err_tok(),
                            instr_name.style(instr())
                        );
                    }
                } else {
                    println!(
                        "{} `{instr_name}` is not a valid instruction name.",
                        err_tok()
                    )
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
            [_, "=", tm @ ("term" | "tm" | "ask"), ..] => {
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
            [lval, "<-", "ask", prompt @ ..] => {
                let lval: LVal = lval.parse()?;
                let prompt = prompt.join(" ");
                let answer = self.prompt(&prompt);
                self.lval_set(&lval, &RVal::Symbol(answer))?;
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
                // Display a Prolog term that's been serialized into memory.
                let rval_text: String = rest.join(" ");
                // let term_parser = pentagwam::syntax::Term::parser().then_ignore(end());
                // let term = term_parser.parse::<_, &str>(term_text.as_str())?;
                let rval: RVal = rval_text.parse()?;
                let term_root = self.eval_to_val(&rval)?.try_as_cell_ref(&self.mem)?;
                let term = Term::deserialize(term_root, &self.mem).unwrap();
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

    fn conditional_skip(&mut self, cmd_split: &[&str]) -> Result<ControlFlow<SkipReason>> {
        match cmd_split {
            ["if" | "when", rval1, "==", rval2] => {
                if all_branches_match(&self.branch_stack) {
                    let val1 = self.eval_to_val(&rval1.parse()?)?;
                    let val2 = self.eval_to_val(&rval2.parse()?)?;
                    if val1.dyn_eq(&val2, &self.mem) {
                        self.branch_stack.push((Some(true), Cond::Consequent));
                        println!("=> {}", "Equal.".style(note()));
                    } else {
                        self.branch_stack.push((Some(false), Cond::Consequent));
                        println!("=> {}", "Not equal.".style(note()));
                    }
                } else {
                    self.branch_stack.push((None, Cond::Consequent));
                }
                let depth = self.branch_stack.len();
                println!(
                    "=> {}",
                    format!("Conditional block #{depth} begin.",).style(note())
                );
                Ok(ControlFlow::Break(SkipReason::CmdCompleted))
            }
            ["else"] => {
                if let Some((_b, cond)) = self.branch_stack.last_mut() {
                    *cond = Cond::Alternative;
                    let depth = self.branch_stack.len();
                    println!(
                        "=> {}",
                        format!("Alternative branch for conditional block #{depth}").style(note())
                    );
                    Ok(ControlFlow::Break(SkipReason::CmdCompleted))
                } else {
                    println!("{} No matching `if` or `when` block to `else`.", err_tok());
                    Ok(ControlFlow::Break(SkipReason::Error))
                }
            }
            ["end", ..] => {
                let depth = self.branch_stack.len();
                println!(
                    "=> {}",
                    format!("Conditional block #{depth} end.").style(note())
                );

                if self.branch_stack.pop().is_none() {
                    println!("{} No matching `if` or `when` block to `end`.", err_tok());
                }
                Ok(ControlFlow::Break(SkipReason::CmdCompleted))
            }
            _regular_cmd => {
                if all_branches_match(&self.branch_stack) {
                    Ok(ControlFlow::Continue(()))
                } else {
                    Ok(ControlFlow::Break(SkipReason::CmdSkipped))
                }
            }
        }
    }
}

fn all_branches_match(stack: &[(Option<bool>, Cond)]) -> bool {
    stack.iter().all(|pair| match pair {
        (None, _) => false,
        (Some(true), Cond::Consequent) | (Some(false), Cond::Alternative) => true,
        (Some(true), Cond::Alternative) | (Some(false), Cond::Consequent) => false,
    })
}

enum SkipReason {
    CmdCompleted,
    CmdSkipped,
    Error,
}
