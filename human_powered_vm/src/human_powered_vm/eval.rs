use pentagwam::cell::Cell;

use crate::human_powered_vm::FieldData;

use super::{
    error::{Error, Result},
    vals::{cellval::CellVal, lval::LVal, rval::RVal, val::Val, valty::ValTy},
    HumanPoweredVm,
};

impl HumanPoweredVm {
    pub(super) fn eval_to_val(&self, rval: &RVal) -> Result<Val> {
        match rval {
            RVal::Deref(inner) => {
                let val = self.eval_to_val(inner)?;
                let cell_ref = val.try_as_cell_ref_like()?;
                self.mem
                    .try_cell_read(cell_ref)
                    .map(Val::Cell)
                    .ok_or(Error::OutOfBoundsMemRead(cell_ref))
            }
            RVal::Index(base, offset) => {
                let base = self.eval_to_val(base)?.try_as_cell_ref_like()?;
                let offset = self.eval_to_val(offset)?.try_as_usize()?;
                let addr = base + offset;
                self.mem
                    .try_cell_read(addr)
                    .map(Val::Cell)
                    .ok_or(Error::OutOfBoundsMemRead(addr))
            }
            RVal::Usize(u) => Ok(Val::Usize(*u)),
            RVal::I32(i) => Ok(Val::I32(*i)),
            RVal::Symbol(s) => Ok(Val::Symbol(s.clone())),
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

    pub(super) fn eval_cellval_to_cell(&self, cell: &CellVal) -> Result<Cell> {
        Ok(match cell {
            CellVal::Ref(r) => Cell::Ref(self.eval_to_val(r)?.try_as_cell_ref()?),
            CellVal::Rcd(r) => Cell::Rcd(self.eval_to_val(r)?.try_as_cell_ref()?),
            CellVal::Lst(r) => Cell::Lst(self.eval_to_val(r)?.try_as_cell_ref()?),
            CellVal::Int(i) => Cell::Int(self.eval_to_val(i)?.try_as_i32()?),
            CellVal::Sym(s) => {
                let val = self.eval_to_val(s)?;
                let text = val.try_as_symbol()?;
                Cell::Sym(self.intern_sym(text))
            }
            CellVal::Sig { fname, arity } => Cell::Sig(self.mem.intern_functor(fname, *arity)),
            CellVal::Nil => Cell::Nil,
        })
    }

    pub(super) fn lval_set(&mut self, lval: &LVal, rval: &RVal) -> Result<Val> {
        let rhs = self.eval_to_val(rval)?;
        match &lval {
            // @123.* <- <rval>
            // Ref(@123).* <- <rval>
            LVal::Deref(inner) => {
                let inner = self.eval_to_val(inner)?;
                let r = inner.try_as_cell_ref_like()?;
                if rhs.ty() != ValTy::Cell {
                    return Err(Error::AssignmentTypeError {
                        expected: "Cell".into(),
                        received: rhs.ty(),
                    });
                }
                self.mem
                    .try_cell_write(r, rhs.try_as_cell()?)
                    .ok_or(Error::OutOfBoundsMemWrite(r))?;
            }

            // arr[123] <- <rval>
            LVal::Index(base, offset) => {
                let base = self.eval_to_val(base)?.try_as_cell_ref()?;
                let offset = self.eval_to_val(offset)?.try_as_usize()?;
                let addr = base + offset;
                self.mem
                    .try_cell_write(addr, rhs.try_as_cell()?)
                    .ok_or(Error::OutOfBoundsMemWrite(addr))?;
            }

            // instr_ptr <- <rval>
            LVal::InstrPtr => self.instr_ptr = rhs.try_as_usize()?,

            // some_field <- <rval>
            // some_field_alias <- <rval>
            LVal::Field(field) => {
                if let Some(fdata) = self.fields.get_mut(field) {
                    fdata.assign_val(rhs.clone())?;
                    println!("Wrote `{}` to `{field}`.", self.mem.display(&rhs));
                } else if let Some((base_name, fdata)) = self
                    .fields
                    .iter_mut()
                    .find(|(_base_name, fdata)| fdata.aliases.contains(field))
                {
                    fdata.assign_val(rhs.clone())?;
                    println!(
                        "Wrote `{}` to `{field}` (alias of `{base_name}`).",
                        self.mem.display(&rhs)
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
                        self.mem.display(&rhs)
                    );
                }
            }

            // .tmp_var <- <rval>
            // .tmp_var_alias <- <rval>
            LVal::TmpVar(var_name) => {
                if let Some(fdata) = self.tmp_vars.get_mut(var_name) {
                    fdata.assign_val(rhs.clone())?;
                    println!("Wrote `{}` to `.{var_name}`.", self.mem.display(&rhs));
                } else if let Some((base_name, fdata)) = self
                    .tmp_vars
                    .iter_mut()
                    .find(|(_base_name, fdata)| fdata.aliases.contains(var_name))
                {
                    fdata.assign_val(rhs.clone())?;
                    println!(
                        "Wrote `{}` to `.{var_name}` (alias of `.{base_name}`).",
                        self.mem.display(&rhs)
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
                        self.mem.display(&rhs)
                    );
                }
            }
        }

        Ok(rhs)
    }
}
