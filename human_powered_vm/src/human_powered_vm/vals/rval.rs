use crate::human_powered_vm::error::{Error, Result};

use crate::human_powered_vm::vals::cellval::CellVal;

use chumsky::prelude::*;
use derive_more::From;
use pentagwam::defs::CellRef;
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

            let index = rval
                .clone()
                .then(rval.clone())
                .delimited_by(just("["), just("]"))
                .map(|(base, offset)| RVal::Index(Box::new(base), Box::new(offset)));

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

            deref
                .or(index)
                .or(instr_ptr)
                .or(cell_ref_lit)
                .or(usize_lit)
                .or(i32_lit)
                .or(tmp_var)
                // .or(CellVal::parser().map(RVal::Cell))
                .or(field)
        })
    }
}

impl FromStr for RVal {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self::parser().parse(s)?)
    }
}
