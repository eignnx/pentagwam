use crate::human_powered_vm::error::{Error, Result};

use crate::human_powered_vm::vals::cellval::CellVal;

use chumsky::prelude::*;
use derive_more::From;
use pentagwam::defs::CellRef;
use pentagwam::mem::Mem;
use std::{fmt, str::FromStr};

use super::valty::ValTy;

#[derive(Debug, From, Clone)]
pub enum RVal {
    Deref(Box<RVal>),
    Index(Box<RVal>, Box<RVal>),
    #[from]
    CellRef(CellRef),
    Usize(usize),
    I32(i32),
    Field(String),
    TmpVar(String),
    InstrPtr,
    Cell(Box<CellVal>),
}

impl Default for RVal {
    fn default() -> Self {
        Self::Usize(0)
    }
}

impl fmt::Display for RVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RVal::Deref(inner) => write!(f, "*{inner}"),
            RVal::Index(base, offset) => write!(f, "{base}[{offset}]"),
            RVal::CellRef(cell_ref) => write!(f, "{cell_ref}"),
            RVal::Usize(u) => write!(f, "{u}"),
            RVal::I32(i) => write!(f, "{i:+}"),
            RVal::Field(field) => write!(f, "self.{field}"),
            RVal::TmpVar(name) => write!(f, ".{name}"),
            RVal::InstrPtr => write!(f, "self.instr_ptr"),
            RVal::Cell(cell) => write!(f, "{cell:?}"),
        }
    }
}

impl RVal {
    pub fn ty(&self) -> ValTy {
        match self {
            RVal::Deref(_) => ValTy::AnyCellVal,
            RVal::Index(..) => ValTy::AnyCellVal,
            RVal::CellRef(_) => ValTy::CellRef,
            RVal::Usize(_) => ValTy::Usize,
            RVal::I32(_) => ValTy::I32,
            RVal::Field(field) => ValTy::TypeOf(field.clone()),
            RVal::TmpVar(name) => ValTy::TypeOf(name.clone()),
            RVal::InstrPtr => ValTy::Usize,
            RVal::Cell(_) => ValTy::AnyCellVal,
        }
    }

    pub fn parser() -> impl Parser<char, Self, Error = Simple<char>> {
        recursive::<_, _, _, _, Simple<char>>(|rval| {
            let deref = just("*")
                .ignore_then(rval.clone())
                .map(|inner| RVal::Deref(Box::new(inner)));

            let instr_ptr = just("instr_ptr").or(just("ip")).map(|_| RVal::InstrPtr);

            let cell_ref_lit = just("@")
                .ignore_then(text::digits(10))
                .try_map(|s: String, span| s.parse::<usize>().map_err(|e| Simple::custom(span, e)))
                .map(|u| RVal::CellRef(CellRef::new(u)));

            let usize_lit = text::digits(10)
                .try_map(|s: String, span| s.parse::<usize>().map_err(|e| Simple::custom(span, e)))
                .map(RVal::Usize);

            let i32_lit = text::digits(10)
                .try_map(|s: String, span| s.parse::<i32>().map_err(|e| Simple::custom(span, e)))
                .map(RVal::I32);

            let tmp_var = just(".").ignore_then(text::ident()).map(RVal::TmpVar);

            let field = text::ident().map(RVal::Field);

            let cell_lit = CellVal::parser(rval.clone()).map(Box::new).map(RVal::Cell);

            let atomic = choice((
                instr_ptr,
                cell_ref_lit,
                usize_lit,
                i32_lit,
                tmp_var,
                field,
                cell_lit,
            ));

            let index = atomic
                .clone()
                .then(
                    rval.clone()
                        .delimited_by(just("["), just("]"))
                        .repeated()
                        .at_least(1),
                )
                .foldl(|a, b| RVal::Index(Box::new(a), Box::new(b)));

            choice((deref, index, atomic))
        })
    }
}

impl FromStr for RVal {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self::parser().parse(s)?)
    }
}

pub struct RValFmt<'a> {
    rval: &'a RVal,
    mem: &'a Mem,
}

impl RVal {
    pub fn display<'a>(&'a self, mem: &'a Mem) -> RValFmt<'a> {
        RValFmt { rval: self, mem }
    }
}

impl std::fmt::Display for RValFmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.rval {
            RVal::Deref(inner) => write!(f, "*{}", inner.display(self.mem)),
            RVal::Index(base, offset) => write!(
                f,
                "{}[{}]",
                base.display(self.mem),
                offset.display(self.mem),
            ),
            RVal::CellRef(r) => write!(f, "{r}"),
            RVal::Usize(u) => write!(f, "{u}"),
            RVal::I32(i) => write!(f, "{i:+}"),
            RVal::Field(field) => write!(f, "self.{field}"),
            RVal::TmpVar(name) => write!(f, ".{name}"),
            RVal::InstrPtr => write!(f, "instr_ptr"),
            RVal::Cell(cell) => write!(f, "{}", cell.display()),
        }
    }
}
