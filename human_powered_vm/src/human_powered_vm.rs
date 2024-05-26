use derive_more::From;
use pentagwam::{
    bc::instr::Instr,
    cell::{Cell, Functor},
    defs::CellRef,
    mem::Mem,
};
use std::{
    collections::HashMap,
    io::{Read, Write},
    ops::ControlFlow,
};

use crate::human_powered_vm::{
    error::{Error, Result},
    vals::{LVal, RVal},
};

use self::vals::{CellVal, Val};

pub mod error;
pub mod vals;

pub struct HumanPoweredVm {
    fields: HashMap<String, Val>,
    instr_ptr: usize,
    mem: Mem,
}

const SAVE_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/FIELDS.txt");

impl Drop for HumanPoweredVm {
    fn drop(&mut self) {
        // save all field names to the save file:
        let mut file = std::fs::File::create(SAVE_FILE).unwrap();
        for field in self.fields.keys() {
            writeln!(file, "{field}").unwrap();
        }
    }
}

impl HumanPoweredVm {
    fn init_fields() -> Result<HashMap<String, Val>> {
        let mut fields = HashMap::new();

        match std::fs::File::open(SAVE_FILE) {
            Ok(mut file) => {
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                for line in buf.lines() {
                    let field = line.trim();
                    if field.contains(char::is_whitespace) {
                        return Err(Error::BadSaveFileFormat);
                    }
                    fields.insert(field.to_owned(), Default::default());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!(
                    "No FIELDS.txt save file found. On exit, one will be created at: {SAVE_FILE}"
                );
            }
            Err(e) => return Err(e)?,
        };

        Ok(fields)
    }
    pub fn new() -> Result<Self> {
        Ok(Self {
            fields: Self::init_fields()?,
            instr_ptr: 0,
            mem: Mem::new(),
        })
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
            if let Some(instr) = program.get(self.instr_ptr) {
                println!("instr #{:04}: {instr:?}", self.instr_ptr);
                println!();
            } else {
                println!("[Instruction pointer beyond end of program]");
                println!();
            }

            let cmd = self.prompt("Enter a command");
            let cmd_split = cmd.split_whitespace().collect::<Vec<_>>();
            match self.handle_cmd(&cmd_split) {
                Ok(ControlFlow::Break(())) => break,
                Ok(ControlFlow::Continue(())) => continue,
                Err(e) => println!("!> {e}\n"),
            }
        }
        Ok(())
    }

    fn handle_cmd(&mut self, cmd: &[&str]) -> Result<ControlFlow<()>> {
        match cmd {
            [] | ["help" | "h" | "?" | "--help"] => self.print_help(),
            ["quit" | "q"] => return Ok(ControlFlow::Break(())),
            ["fields" | "f"] => {
                println!("Virtual Machine Fields: {{");
                for (field, value) in self.fields.iter() {
                    println!("    {field}: {value},");
                }
                println!("}}");
                println!();
            }
            ["next" | "n"] => {
                self.instr_ptr += 1;
            }
            ["p", rval] => {
                self.print_rval(rval)?;
            }
            ["del", field_name] => {
                if self.fields.remove(*field_name).is_some() {
                    println!("Deleted field `{field_name}`.");
                } else {
                    println!("Field `{field_name}` can't be deleted because it doesn't exist.")
                }
                println!();
            }
            [lval, "=", rhs] => {
                self.assign_to_lval(lval, rhs)?;
            }
            _ => println!("=> Unknown command `{}`.", cmd.join(" ")),
        }
        Ok(ControlFlow::Continue(()))
    }

    fn eval_to_val(&self, rval: &RVal) -> Result<Val> {
        match rval {
            RVal::Usize(u) => Ok(Val::Usize(*u)),
            RVal::I32(i) => Ok(Val::I32(*i)),
            RVal::Cell(c) => Ok(Val::Cell(self.eval_cellval_to_cell(c)?)),
            RVal::CellRef(r) => {
                let cell = self
                    .mem
                    .try_cell_read(*r)
                    .ok_or(Error::OutOfBoundsMemRead(*r))?;
                Ok(Val::Cell(cell))
            }
            RVal::Field(field) => Ok(self
                .fields
                .get(field)
                .cloned()
                .ok_or_else(|| Error::UndefinedField(field.clone()))?),
            RVal::InstrPtr => Ok(Val::Usize(self.instr_ptr)),
        }
    }

    fn eval_cellval_to_cell(&self, cell: &CellVal<RVal>) -> Result<Cell> {
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
        let rval = self.eval_to_val(&rval_name.parse()?)?;
        println!("=> {rval_name}: {rval}");
        Ok(())
    }

    fn lval_set(&mut self, lval: &LVal, rval: &RVal) -> Result<()> {
        let rhs = self.eval_to_val(rval)?;
        match &lval {
            LVal::InstrPtr => self.instr_ptr = rhs.expect_usize()?,
            LVal::Field(field) => {
                let _ = self
                    .fields
                    .entry(field.clone())
                    .and_modify(|v| *v = rhs.clone())
                    .or_insert_with(|| {
                        println!("Creating new field `{lval}`.");
                        rhs.clone()
                    });
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
                        received: rval.ty().into(),
                    })
                }
            },
        }
        Ok(())
    }

    fn assign_to_lval(&mut self, lval_name: &str, rhs_name: &str) -> Result<()> {
        let lval = lval_name.parse()?;
        let rval = rhs_name.parse()?;
        self.lval_set(&lval, &rval)?;
        println!("Wrote `{rval}` to `{lval}`.");
        println!();
        Ok(())
    }

    fn print_help(&self) {
        println!();
        println!("Commands:");
        println!("  p <rval>         - Print the value of <rval>.");
        println!("  <lval> = <rval>  - Assign the value of <rval> to <lval>.");
        println!("  fields | f       - Print all the data fields of the VM.");
        println!("  next | n         - Advance to the next instruction.");
        println!("  q | quit         - Quit the program.");
        println!("  h | help         - Print this help message.");
        println!();
    }
}
