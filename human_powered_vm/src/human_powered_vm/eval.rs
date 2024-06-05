use pentagwam::{cell::Cell, defs::CellRef};

use crate::human_powered_vm::FieldData;

use super::{
    error::{Error, Result},
    vals::{
        cellval::CellVal,
        lval::LVal,
        rval::RVal,
        slice::{Region, Slice},
        val::Val,
        valty::ValTy,
    },
    HumanPoweredVm,
};

impl HumanPoweredVm {
    pub(super) fn eval_to_val(&self, rval: &RVal) -> Result<Val> {
        match rval {
            RVal::AddressOf(inner) => self.eval_address_of(inner),
            RVal::Deref(inner) => {
                let val = self.eval_to_val(inner)?;
                let cell_ref = val.try_as_cell_ref_like()?;
                self.mem
                    .try_cell_read(cell_ref)
                    .map(Val::Cell)
                    .ok_or(Error::OutOfBoundsMemRead(Region::Mem, cell_ref.usize()))
            }
            RVal::Index(base, offset) => {
                let base = self.eval_to_val(base)?.try_as_cell_ref_like()?;
                let offset = self.eval_to_val(offset)?.try_as_any_int()?;
                let addr = CellRef::from((base.i64() + offset) as usize);
                self.mem
                    .try_cell_read(addr)
                    .map(Val::Cell)
                    .ok_or(Error::OutOfBoundsMemRead(Region::Mem, addr.usize()))
            }
            RVal::IndexSlice(base, start, len) => {
                self.eval_index_slice(base, start.as_deref(), len.as_deref())
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
        }
    }

    fn eval_index_slice(
        &self,
        base: &RVal,
        start: Option<&RVal>,
        len: Option<&RVal>,
    ) -> Result<Val> {
        let start = self.try_eval_as_slice_bound(start)?;
        let len = self.try_eval_as_slice_bound(len)?;

        let base_slice: Slice<usize> = match self.eval_to_val(base)? {
            Val::CellRef(base) => Slice {
                region: Region::Mem,
                start: base.usize(),
                len: 0,
            },

            Val::Usize(base) => Slice {
                region: Region::Code,
                start: base,
                len: 0,
            },

            Val::Slice(slice) => slice,

            other => {
                return Err(Error::UnsliceableValue(
                    self.mem.display(&other).to_string(),
                ))
            }
        };

        Ok(Val::Slice(Slice::normalized_from(base_slice, start, len)?))
    }

    fn eval_address_of(&self, inner: &RVal) -> Result<Val> {
        match inner {
            RVal::Deref(inner) => Ok(Val::CellRef(
                self.eval_to_val(inner.as_ref())?.try_as_cell_ref()?,
            )),
            RVal::Index(base, offset) => {
                let base = self.eval_to_val(base)?.try_as_cell_ref_like()?;
                let offset = self.eval_to_val(offset)?.try_as_any_int()?;
                let cell_ref = (base.i64() + offset) as usize;
                Ok(Val::CellRef(cell_ref.into()))
            }
            RVal::IndexSlice(base, start, _len) => {
                let start = self.try_eval_as_slice_bound(start.as_deref())?;
                match self.eval_to_val(base)? {
                    Val::CellRef(base) => {
                        let addr = (base.usize() as i64 + start.unwrap_or(0)) as usize;
                        Ok(Val::CellRef(addr.into()))
                    }
                    Val::Usize(base) => {
                        let addr = (base as i64 + start.unwrap_or(0)) as usize;
                        Ok(Val::Usize(addr))
                    }
                    other => Err(Error::UnsliceableValue(
                        self.mem.display(&other).to_string(),
                    )),
                }
            }
            RVal::AddressOf(_) => Err(Error::BadAddressOfArgument {
                reason: "Can't take the address of an address-of expression.",
                value: self.mem.display(inner).to_string(),
            }),
            RVal::CellRef(_) => Err(Error::BadAddressOfArgument {
                reason: "Can't take the address of a cell reference literal \
                         because that is still just a temporary; it lives \
                         nowhere.",
                value: self.mem.display(inner).to_string(),
            }),
            RVal::Usize(_) | RVal::I32(_) | RVal::Symbol(_) | RVal::Cell(_) => {
                Err(Error::BadAddressOfArgument {
                    reason: "Can't take the address of a temporary value.",
                    value: self.mem.display(inner).to_string(),
                })
            }
            RVal::Field(_) | RVal::TmpVar(_) => Err(Error::BadAddressOfArgument {
                reason: "Can't take the address of a field or temp var \
                                 because those won't exist at runtime (they're \
                                 just for the human-powered VM).",
                value: self.mem.display(inner).to_string(),
            }),
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
                    .ok_or(Error::OutOfBoundsMemWrite(Region::Mem, r.usize()))?;
            }

            // arr[123] <- <rval>
            LVal::Index(base, offset) => {
                let base = self.eval_to_val(base)?.try_as_cell_ref()?;
                let offset = self.eval_to_val(offset)?.try_as_usize()?;
                let addr = base + offset;
                self.mem
                    .try_cell_write(addr, rhs.try_as_cell()?)
                    .ok_or(Error::OutOfBoundsMemWrite(Region::Mem, addr.usize()))?;
            }

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
                            default: None,
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
                            default: None,
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

    fn try_eval_as_slice_bound(&self, rval_opt: Option<&RVal>) -> Result<Option<i64>> {
        let Some(rval) = rval_opt.as_ref() else {
            return Ok(None);
        };
        let val = self.eval_to_val(rval)?;
        Ok(Some(val.try_as_any_int()?))
    }
}
