use owo_colors::OwoColorize;
use pentagwam::{cell::Cell, defs::CellRef};

use super::{
    error::{Error, Result},
    FieldData, HumanPoweredVm,
};
use crate::{
    human_powered_vm::styles::{self, name, val, valty},
    vals::{
        cellval::CellVal,
        lval::LVal,
        rval::RVal,
        slice::{Idx, Len, Region, Slice},
        val::Val,
        valty::ValTy,
    },
};

impl HumanPoweredVm {
    pub(super) fn eval_to_val(&self, rval: &RVal) -> Result<Val> {
        match rval {
            RVal::AddressOf(inner) => self.eval_address_of(inner),
            RVal::Deref(inner) => {
                let val = self.eval_to_val(inner)?;
                let cell_ref = val.try_as_cell_ref(&self.mem)?;
                self.mem
                    .try_cell_read(cell_ref)
                    .map(Val::Cell)
                    .ok_or(Error::OutOfBoundsMemRead(Region::Mem, cell_ref.usize()))
            }
            RVal::Index(base, offset) => self.eval_index(base, offset),
            RVal::IndexSlice(base, slice) => self.eval_index_slice(base, slice.as_ref()),
            RVal::Usize(u) => Ok(Val::Usize(*u)),
            RVal::I32(i) => Ok(Val::I32(*i)),
            RVal::Symbol(s) => Ok(Val::Symbol(s.clone())),
            RVal::Cell(c) => Ok(Val::Cell(self.eval_cellval_to_cell(c)?)),
            RVal::CellRef(r) => Ok(Val::CellRef(*r)),
            RVal::Field(field) => {
                if let Some(fdata) = self.save.fields.get(field) {
                    Ok(fdata.value.clone())
                } else {
                    // check aliases:
                    self.save
                        .fields
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
            RVal::InstrParam(idx) => {
                let param = self.instr_param(*idx)?;
                self.eval_to_val(&param)
            }
            RVal::Functor(fname, arity) => Ok(Val::Functor {
                sym: self
                    .eval_to_val(fname)?
                    .try_as_symbol(&self.mem)?
                    .to_string(),
                arity: self.eval_to_val(arity)?.try_as_usize(&self.mem)? as u8,
            }),
        }
    }

    fn eval_index(&self, base: &RVal, offset: &Idx<RVal>) -> std::prelude::v1::Result<Val, Error> {
        let base = self.eval_to_val(base)?;

        let (region, base) = if let Ok(cell_ref) = base.try_as_cell_ref(&self.mem) {
            (Region::Mem, cell_ref.i64())
        } else if let Ok(i) = base.try_as_any_int(&self.mem) {
            (Region::Code, i)
        } else if let Val::Slice { region, start, .. } = base {
            (region, start as i64)
        } else {
            return Err(Error::TypeError {
                expected: "CellRef or Usize or I32".into(),
                received: base.ty(),
                expr: self.mem.display(&base).to_string(),
            });
        };

        let addr_i64 = match offset {
            Idx::Lo => base,
            Idx::Hi => match region {
                Region::Code => self.program.len() as i64,
                Region::Mem => self.mem.heap.len() as i64,
            },
            Idx::Int(idx_rval) => {
                let val = self.eval_to_val(idx_rval)?;
                let int = val.try_as_any_int(&self.mem)?;
                int + base
            }
        };

        let addr_usize =
            usize::try_from(addr_i64).map_err(|_| Error::BelowBoundsSliceStart(addr_i64))?;

        self.mem
            .try_cell_read(addr_usize)
            .map(Val::Cell)
            .ok_or(Error::OutOfBoundsMemRead(Region::Mem, addr_usize))
    }

    fn eval_index_slice(&self, base: &RVal, slice: &Slice<RVal>) -> Result<Val> {
        let Slice { idx, len } = slice
            .map_int(|rval| self.eval_to_val(rval))?
            .map_int(|val| val.try_as_any_int(&self.mem))?;

        let base_val = self.eval_to_val(base)?;

        let (region, start) =
            if let Ok(Val::CellRef(base)) = base_val.try_convert(ValTy::CellRef, &self.mem) {
                (Region::Mem, idx + base.i64())
            } else if let Ok(Val::Usize(base)) = base_val.try_convert(ValTy::Usize, &self.mem) {
                (Region::Code, idx + base as i64)
            } else if let slice @ Val::Slice { .. } = base_val {
                // TODO:
                return Err(Error::UnsliceableValue(
                    self.mem.display(&slice).to_string(),
                ));
            } else {
                return Err(Error::UnsliceableValue(self.mem.display(&base).to_string()));
            };

        let start = match start {
            Idx::Lo => 0,
            Idx::Hi => match region {
                Region::Code => self.program.len(),
                Region::Mem => self.mem.heap.len(),
            },
            Idx::Int(idx) => usize::try_from(idx).map_err(|_| Error::BelowBoundsSliceStart(idx))?,
        };

        let (start, len) = match len {
            Len::NegInf => (0, start),
            Len::PosInf => {
                let max_len = match region {
                    Region::Code => self.program.len(),
                    Region::Mem => self.mem.heap.len(),
                };

                let len_from_start_to_inf = max_len
                    .checked_sub(start)
                    .ok_or(Error::OutOfBoundsMemRead(region, start))?;

                (start, len_from_start_to_inf)
            }
            Len::Int(len) => {
                if len < 0 {
                    // Negative length means slice backwards from the starting point.
                    let new_start = usize::try_from(start as i64 - len.abs())
                        .map_err(|_| Error::BelowBoundsSliceStart(start as i64 - len.abs()))?;
                    (new_start, len.unsigned_abs() as usize)
                } else {
                    let len = len as usize;
                    (start, len)
                }
            }
        };

        Ok(Val::Slice { region, start, len })
    }

    fn eval_address_of(&self, inner: &RVal) -> Result<Val> {
        match inner {
            RVal::Deref(inner) => Ok(Val::CellRef(
                self.eval_to_val(inner.as_ref())?
                    .try_as_cell_ref(&self.mem)?,
            )),
            RVal::Index(base, offset) => {
                let base = self.eval_to_val(base)?;
                let (region, base) = if let Ok(cell_ref) = base.try_as_cell_ref(&self.mem) {
                    (Region::Mem, cell_ref.i64())
                } else if let Ok(i) = base.try_as_any_int(&self.mem) {
                    (Region::Code, i)
                } else if let slice @ Val::Slice { .. } = base {
                    // (region, start as i64)
                    return Err(Error::BadAddressOfArgument {
                        reason: "operator `.&` is not implemented for slices yet.",
                        value: slice.to_string(),
                    });
                } else {
                    return Err(Error::BadAddressOfArgument {
                        reason: "\
                            operator `.&` can only be applied to a cell \
                            reference (like `@123`), a code address (like \
                            `123`), or a Cell containing one of the previous \
                            two types of values.\
                        ",
                        value: self.mem.display(&base).to_string(),
                    });
                };

                let offset = offset
                    .map_int(|rval| self.eval_to_val(rval))?
                    .map_int(|val| val.try_as_any_int(&self.mem))?;

                match region {
                    Region::Mem => {
                        let addr = match offset {
                            Idx::Lo => usize::try_from(base)
                                .map_err(|_| Error::BelowBoundsSliceStart(base))?,
                            Idx::Hi => self.mem.heap.len(),
                            Idx::Int(idx) => usize::try_from(idx + base)
                                .map_err(|_| Error::BelowBoundsSliceStart(idx + base))?,
                        };
                        Ok(Val::CellRef(addr.into()))
                    }
                    Region::Code => {
                        let addr = match offset {
                            Idx::Lo => base as usize,
                            Idx::Hi => self.program.len(),
                            Idx::Int(idx) => (idx + base)
                                .try_into()
                                .map_err(|_| Error::BelowBoundsSliceStart(idx + base))?,
                        };
                        Ok(Val::Usize(addr))
                    }
                }
            }
            RVal::IndexSlice(base, slice) => {
                let start = slice
                    .idx
                    .map_int(|rval| self.eval_to_val(rval))?
                    .map_int(|val| val.try_as_any_int(&self.mem))?;

                match self.eval_to_val(base)? {
                    Val::CellRef(base) => {
                        // Region::Mem
                        let addr = match start {
                            Idx::Lo => base,
                            Idx::Hi => self.mem.heap.len().into(),
                            Idx::Int(idx) => usize::try_from(idx + base.i64())
                                .map_err(|_| Error::BelowBoundsSliceStart(idx + base.i64()))?
                                .into(),
                        };
                        Ok(Val::CellRef(addr))
                    }
                    Val::Usize(base) => {
                        // Region::Code
                        let addr = match start {
                            Idx::Lo => base,
                            Idx::Hi => self.program.len(),
                            Idx::Int(idx) => (idx + base as i64)
                                .try_into()
                                .map_err(|_| Error::BelowBoundsSliceStart(idx + base as i64))?,
                        };
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
            RVal::Usize(_)
            | RVal::I32(_)
            | RVal::Symbol(_)
            | RVal::Cell(_)
            | RVal::Functor(_, _) => Err(Error::BadAddressOfArgument {
                reason: "Can't take the address of a temporary value.",
                value: self.mem.display(inner).to_string(),
            }),
            RVal::Field(_) | RVal::TmpVar(_) | RVal::InstrParam(_) => {
                Err(Error::BadAddressOfArgument {
                    reason: "Can't take the address of a field, temp var, or \
                            instruction parameter because those won't exist at \
                            runtime (they're just for the human-powered VM).",
                    value: self.mem.display(inner).to_string(),
                })
            }
        }
    }

    pub(super) fn eval_cellval_to_cell(&self, cell: &CellVal) -> Result<Cell> {
        Ok(match cell {
            CellVal::Ref(r) => Cell::Ref(self.eval_to_val(r)?.try_as_cell_ref(&self.mem)?),
            CellVal::Rcd(r) => Cell::Rcd(self.eval_to_val(r)?.try_as_cell_ref(&self.mem)?),
            CellVal::Lst(r) => Cell::Lst(self.eval_to_val(r)?.try_as_cell_ref(&self.mem)?),
            CellVal::Int(i) => Cell::Int(self.eval_to_val(i)?.try_as_i32(&self.mem)?),
            CellVal::Sym(s) => {
                let val = self.eval_to_val(s)?;
                let text = val.try_as_symbol(&self.mem)?;
                Cell::Sym(self.intern_sym(&text))
            }
            CellVal::Sig(functor) => {
                let val = self.eval_to_val(functor)?;
                let (fname, arity) = val.try_as_functor(&self.mem)?;
                Cell::Sig(self.mem.intern_functor(fname, arity))
            }
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
                let r = inner.try_as_cell_ref(&self.mem)?;
                let Val::Cell(rhs) = rhs.try_convert(ValTy::Cell(None), &self.mem)? else {
                    unreachable!()
                };
                self.mem
                    .try_cell_write(r, rhs)
                    .ok_or(Error::OutOfBoundsMemWrite(Region::Mem, r.usize()))?;
                println!(
                    "Wrote `{}` to `{}`.",
                    self.mem.display(&rhs).style(val()),
                    r.style(styles::lval())
                );
            }

            // arr[123] <- <rval>
            LVal::Index(base, offset) => {
                // Note: using `try_as_cell_ref` because program memory can't be written to (currently?).
                let base = self.eval_to_val(base)?.try_as_cell_ref(&self.mem)?;
                let offset = self.eval_to_val(offset)?.try_as_any_int(&self.mem)?;
                let addr: CellRef = usize::try_from(base.i64() + offset)
                    .map_err(|_| Error::BelowBoundsSliceStart(base.i64() + offset))?
                    .into();
                let Val::Cell(rhs) = rhs.try_convert(ValTy::Cell(None), &self.mem)? else {
                    unreachable!()
                };
                self.mem
                    .try_cell_write(addr, rhs)
                    .ok_or(Error::OutOfBoundsMemWrite(Region::Mem, addr.usize()))?;
                println!(
                    "Wrote `{}` to `{}`.",
                    self.mem.display(&rhs).style(val()),
                    addr.style(styles::lval())
                );
            }

            // some_field <- <rval>
            // some_field_alias <- <rval>
            LVal::Field(field) => {
                if let Some(fdata) = self.save.fields.get_mut(field) {
                    fdata.assign_val(rhs.clone(), &self.mem)?;
                    println!(
                        "Wrote `{}` to `{}`.",
                        self.mem.display(&rhs).style(val()),
                        field.style(name())
                    );
                } else if let Some((base_name, fdata)) = self
                    .save
                    .fields
                    .iter_mut()
                    .find(|(_base_name, fdata)| fdata.aliases.contains(field))
                {
                    fdata.assign_val(rhs.clone(), &self.mem)?;
                    println!(
                        "Wrote `{rhs}` to `{alias}` (alias of `{base_name}`).",
                        rhs = self.mem.display(&rhs).style(val()),
                        alias = field.style(name()),
                        base_name = base_name.style(name()),
                    );
                } else {
                    // It must be a new field.
                    self.save.fields.insert(
                        field.to_string(),
                        FieldData {
                            value: rhs.clone(),
                            ty: rhs.ty(),
                            default: None,
                            aliases: Default::default(),
                        },
                    );
                    println!(
                        "Created new field `{}: {} = {}`.",
                        field.style(name()),
                        rhs.ty().style(valty()),
                        self.mem.display(&rhs).style(val())
                    );
                }
            }

            // .tmp_var <- <rval>
            // .tmp_var_alias <- <rval>
            LVal::TmpVar(var_name) => {
                let dot_name = format!(".{var_name}");
                if let Some(fdata) = self.tmp_vars.get_mut(var_name) {
                    fdata.assign_val(rhs.clone(), &self.mem)?;
                    println!(
                        "Wrote `{}` to `{}`.",
                        self.mem.display(&rhs).style(val()),
                        dot_name.style(name())
                    );
                } else if let Some((base_name, fdata)) = self
                    .tmp_vars
                    .iter_mut()
                    .find(|(_base_name, fdata)| fdata.aliases.contains(var_name))
                {
                    fdata.assign_val(rhs.clone(), &self.mem)?;
                    println!(
                        "Wrote `{}` to `{}` (alias of `{}`).",
                        self.mem.display(&rhs).style(val()),
                        dot_name.style(name()),
                        format!(".{base_name}").style(name()),
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
                        "Created new temporary variable `{}: {} = {}`.",
                        dot_name.style(name()),
                        rhs.ty().style(valty()),
                        self.mem.display(&rhs).style(val())
                    );
                }
            }
        }

        Ok(rhs)
    }
}
