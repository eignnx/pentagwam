use derive_more::From;
use pentagwam::{
    bc::instr::Instr,
    cell::{Cell, Functor},
    defs::{CellRef, Sym},
    mem::Mem,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Read, Write},
    ops::ControlFlow,
};

use crate::human_powered_vm::{
    error::{Error, Result},
    vals::{cellval::CellVal, lval::LVal, rval::RVal, val::Val, valty::ValTy},
};

pub mod error;
pub mod instr_fmt;
pub mod val_fmt;
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
                    "No FIELDS.txt save file found. On exit, one will be \
                     created at: {SAVE_FILE}"
                );
                Ok(Default::default())
            }
            Err(e) => Err(e.into()),
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

    pub fn run(&mut self, program: &[Instr<Functor>]) -> Result<()> {
        loop {
            println!();
            if let Some(instr) = program.get(self.instr_ptr) {
                println!(
                    "instr #{:04}: {}",
                    self.instr_ptr,
                    instr_fmt::display_instr(instr, &self.mem)
                );
                println!();
            } else {
                println!("[Instruction pointer beyond end of program]");
                println!();
            }

            let cmd = self.prompt("Enter a command");
            let cmd_split = cmd.split_whitespace().collect::<Vec<_>>();
            match self.handle_cmd(&cmd_split, program) {
                Ok(ControlFlow::Break(())) => break,
                Ok(ControlFlow::Continue(())) => continue,
                Err(e) => println!("!> {e}"),
            }
        }
        Ok(())
    }

    fn handle_cmd(&mut self, cmd: &[&str], program: &[Instr<Functor>]) -> Result<ControlFlow<()>> {
        match cmd {
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
                        println!(
                            "{:^80}",
                            instr_fmt::display_instr(instr, &self.mem).to_string()
                        );
                        println!();
                        println!("{docs}");
                        println!("{:-<80}", "");
                    } else {
                        println!(
                            "!> No documentation available instruction `{}`",
                            instr_fmt::display_instr(instr, &self.mem)
                        );
                    }
                }
            }
            ["quit" | "q" | ":wq" | ":q"] => {
                println!("Exiting...");
                return Ok(ControlFlow::Break(()));
            }
            ["fields" | "f"] => {
                println!("Virtual Machine Fields:");
                for (field, fdata) in self.fields.iter() {
                    print!(
                        "\t{field}: {} = {}",
                        fdata.ty,
                        fdata.value.display(&self.mem)
                    );
                    if !fdata.aliases.is_empty() {
                        print!("\t\taliases: ");
                        for (i, alias) in fdata.aliases.iter().enumerate() {
                            print!("{sep}{alias}", sep = if i > 0 { ", " } else { "" });
                        }
                    }
                    println!(";");
                }

                println!();
                println!("Temporary Variables:");
                for (var_name, fdata) in self.tmp_vars.iter() {
                    print!(
                        "\t.{var_name}: {} = {}",
                        fdata.ty,
                        fdata.value.display(&self.mem)
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
            ["push", rval] => {
                let rval: RVal = rval.parse()?;
                let val = self.eval_to_val(&rval)?;
                let cell = val.expect_cell()?;
                self.mem.push(cell);
                println!("Pushed `{}` onto top of heap.", val.display(&self.mem));
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
                    lval.display(&self.mem)
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
                    println!("!> Can't unalias `{alias}` from `{field}` because `{field}` doesn't exist.");
                }
            }
            ["term" | "tm", rval] => {
                // Display a Prolog term
                let rval: RVal = rval.parse()?;
                let val = self.eval_to_val(&rval)?;
                let cell_ref = val.expect_cell_ref()?;
                let disp = self.mem.display_term(cell_ref);
                println!("=> {} == {disp}", rval.display(&self.mem));
            }
            [rval] => {
                self.print_rval(rval)?;
            }
            _ => println!("!> Unknown command `{}`.", cmd.join(" ")),
        }
        Ok(ControlFlow::Continue(()))
    }

    fn program_listing(&self, rest: &[&str], program: &[Instr<Functor>]) -> Result<()> {
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

    fn eval_to_val(&self, rval: &RVal) -> Result<Val> {
        match rval {
            RVal::Deref(inner) => {
                let val = self.eval_to_val(inner)?;
                match val {
                    Val::CellRef(r) | Val::Cell(Cell::Ref(r) | Cell::Rcd(r) | Cell::Lst(r)) => self
                        .mem
                        .try_cell_read(r)
                        .map(Val::Cell)
                        .ok_or(Error::OutOfBoundsMemRead(r)),
                    Val::Cell(Cell::Int(_) | Cell::Sym(_) | Cell::Sig(_) | Cell::Nil)
                    | Val::Usize(_)
                    | Val::I32(_) => Err(Error::TypeError {
                        expected: "CellRef, Ref, Rcd, or Lst".into(),
                        received: val.ty(),
                    }),
                }
            }
            RVal::Index(base, offset) => {
                let base = self.eval_to_val(base)?.expect_cell_ref()?;
                let offset = self.eval_to_val(offset)?.expect_usize()?;
                let addr = base + offset;
                let cell = self
                    .mem
                    .try_cell_read(addr)
                    .ok_or(Error::OutOfBoundsMemRead(addr))?;
                Ok(Val::Cell(cell))
            }
            RVal::Usize(u) => Ok(Val::Usize(*u)),
            RVal::I32(i) => Ok(Val::I32(*i)),
            RVal::Cell(c) => Ok(Val::Cell(self.eval_cellval_to_cell(c)?)),
            RVal::CellRef(r) => Ok(Val::CellRef(*r)),
            RVal::Field(field) => {
                if let Some(fdata) = self.fields.get(field) {
                    Ok(fdata.value.clone())
                } else {
                    // check aliases:
                    self.fields
                        .values()
                        .find_map(|fdata| {
                            if fdata.aliases.contains(field) {
                                Some(fdata.value.clone())
                            } else {
                                None
                            }
                        })
                        .ok_or_else(|| Error::UndefinedField(field.to_string()))
                }
            }
            RVal::TmpVar(name) => {
                if let Some(fdata) = self.tmp_vars.get(name) {
                    Ok(fdata.value.clone())
                } else {
                    self.tmp_vars
                        .values()
                        .find_map(|fdata| {
                            if fdata.aliases.contains(name) {
                                Some(fdata.value.clone())
                            } else {
                                None
                            }
                        })
                        .ok_or_else(|| Error::UndefinedTmpVar(name.to_string()))
                }
            }
            RVal::InstrPtr => Ok(Val::Usize(self.instr_ptr)),
        }
    }

    fn eval_cellval_to_cell(&self, cell: &CellVal) -> Result<Cell> {
        Ok(match cell {
            CellVal::Ref(r) => Cell::Ref(self.eval_to_val(r)?.expect_cell_ref()?),
            CellVal::Rcd(r) => Cell::Rcd(self.eval_to_val(r)?.expect_cell_ref()?),
            CellVal::Lst(r) => Cell::Lst(self.eval_to_val(r)?.expect_cell_ref()?),
            CellVal::Int(i) => Cell::Int(self.eval_to_val(i)?.expect_i32()?),
            CellVal::Sym(text) => Cell::Sym(self.mem.intern_sym(text)),
            CellVal::Sig { fname, arity } => Cell::Sig(self.mem.intern_functor(fname, *arity)),
            CellVal::Nil => Cell::Nil,
        })
    }

    fn print_rval(&self, rval_name: &str) -> Result<()> {
        let val = self.eval_to_val(&rval_name.parse()?)?;
        println!("=> {rval_name} == {}", val.display(&self.mem));
        Ok(())
    }

    fn lval_set(&mut self, lval: &LVal, rval: &RVal) -> Result<Val> {
        let rhs = self.eval_to_val(rval)?;
        match &lval {
            LVal::Deref(inner) => {
                let inner = self.eval_to_val(inner)?;
                match inner {
                    Val::CellRef(r) => {
                        if rhs.ty() != ValTy::AnyCellVal {
                            return Err(Error::AssignmentTypeError {
                                expected: "Cell".into(),
                                received: rhs.ty(),
                            });
                        }
                        self.mem
                            .try_cell_write(r, rhs.expect_cell()?)
                            .ok_or(Error::OutOfBoundsMemWrite(r))?;
                    }
                    Val::Cell(Cell::Ref(r) | Cell::Rcd(r) | Cell::Lst(r)) => {
                        if rhs.ty() != ValTy::AnyCellVal {
                            return Err(Error::AssignmentTypeError {
                                expected: "Cell".into(),
                                received: rhs.ty(),
                            });
                        }
                        self.mem
                            .try_cell_write(r, rhs.expect_cell()?)
                            .ok_or(Error::OutOfBoundsMemWrite(r))?;
                    }
                    Val::Cell(Cell::Int(_) | Cell::Sym(_) | Cell::Sig(_) | Cell::Nil)
                    | Val::I32(_)
                    | Val::Usize(_) => {
                        return Err(Error::AssignmentTypeError {
                            expected: "CellRef, Ref, Rcd, or Lst".into(),
                            received: inner.ty(),
                        })
                    }
                }
            }
            LVal::InstrPtr => self.instr_ptr = rhs.expect_usize()?,
            LVal::Field(field) => {
                if let Some(fdata) = self.fields.get_mut(field) {
                    fdata.assign_val(rhs.clone())?;
                    println!("Wrote `{}` to `{field}`.", rhs.display(&self.mem));
                } else if let Some((base_name, fdata)) = self
                    .fields
                    .iter_mut()
                    .find(|(_base_name, fdata)| fdata.aliases.contains(field))
                {
                    fdata.assign_val(rhs.clone())?;
                    println!(
                        "Wrote `{}` to `{field}` (alias of `{base_name}`).",
                        rhs.display(&self.mem)
                    );
                } else {
                    // It must be a new field.
                    self.fields.insert(
                        field.to_string(),
                        FieldData {
                            value: rhs.clone(),
                            ty: rhs.ty(),
                            aliases: Default::default(),
                        },
                    );
                    println!(
                        "Created new field `self.{field}: {} = {}`.",
                        rhs.ty(),
                        rhs.display(&self.mem)
                    );
                }
            }
            LVal::TmpVar(var_name) => {
                if let Some(fdata) = self.tmp_vars.get_mut(var_name) {
                    fdata.assign_val(rhs.clone())?;
                    println!("Wrote `{}` to `.{var_name}`.", rhs.display(&self.mem));
                } else if let Some((base_name, fdata)) = self
                    .tmp_vars
                    .iter_mut()
                    .find(|(_base_name, fdata)| fdata.aliases.contains(var_name))
                {
                    fdata.assign_val(rhs.clone())?;
                    println!(
                        "Wrote `{}` to `.{var_name}` (alias of `.{base_name}`).",
                        rhs.display(&self.mem)
                    );
                } else {
                    // It must be a new tmp var.
                    self.tmp_vars.insert(
                        var_name.to_string(),
                        FieldData {
                            value: rhs.clone(),
                            ty: rhs.ty(),
                            aliases: Default::default(),
                        },
                    );
                    println!(
                        "Created new temporary variable `.{var_name}: {} = {}`.",
                        rhs.ty(),
                        rhs.display(&self.mem)
                    );
                }
            }
            LVal::CellRef(const_cell_ref) => match rhs {
                Val::Cell(cell) => self
                    .mem
                    .try_cell_write(*const_cell_ref, cell)
                    .ok_or(Error::OutOfBoundsMemWrite(*const_cell_ref))?,
                Val::CellRef(rhs_cell_ref) => {
                    let rhs_cell = self
                        .mem
                        .try_cell_read(rhs_cell_ref)
                        .ok_or(Error::OutOfBoundsMemRead(rhs_cell_ref))?;
                    self.mem
                        .try_cell_write(*const_cell_ref, rhs_cell)
                        .ok_or(Error::OutOfBoundsMemWrite(*const_cell_ref))?;
                }
                Val::Usize(_) | Val::I32(_) => {
                    return Err(Error::AssignmentTypeError {
                        expected: "Cell or CellRef".into(),
                        received: rval.ty(),
                    })
                }
            },
        }
        Ok(rhs)
    }

    fn assign_to_lval(&mut self, lval_name: &str, rhs_name: &str) -> Result<()> {
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

    fn print_help(&self) {
        println!();
        println!("Commands:");
        println!("  <rval>           - Print the value of <rval>.");
        println!("  <lval> <- <rval> - Assign the value of <rval> to <lval>.");
        println!("  <lval> <- tm <tm>");
        println!("                   - Assign the Prolog term <tm> to <lval>.");
        println!("  <rval>           - Print the value of <rval>.");
        println!("  term <rval>      - Print the Prolog term residing in memory");
        println!("                     at CellRef <rval>.");
        println!("  push <rval>      - Push the value of <rval> onto the heap.");
        println!("  fields | f       - Print all the data fields of the VM.");
        println!("  list | l [from|next|prev|last|first <n>]");
        println!("                   - Print a program listing.");
        println!("  docs | doc | d   - Print the documentation for the current");
        println!("                     instruction.");
        println!("  next | n         - Advance to the next instruction.");
        println!("  alias <new> -> <old>");
        println!("                   - Alias <old> as <new>.");
        println!("  unalias <alias> -> <field>");
        println!("                   - Unalias <alias> from <field>.");
        println!("  q | quit         - Quit the program.");
        println!("  h | help         - Print this help message.");
    }

    pub fn intern_sym(&self, text: &str) -> Sym {
        self.mem.intern_sym(text)
    }

    fn populate_default_field_values(&mut self) {
        // We'd like for the Deserialize implementation to look at the `ValTy`
        // of the field and generate a default based on that, but I don't know
        // how to do that. So we'll just post-process a bit.
        for (_field, data) in self.fields.iter_mut() {
            data.value = data.ty.default_val();
        }
    }
}
