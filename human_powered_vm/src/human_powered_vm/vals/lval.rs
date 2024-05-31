use super::rval::RVal;
use crate::human_powered_vm::error::{Error, Result};
use chumsky::prelude::*;
use pentagwam::mem::Mem;
use std::{fmt, str::FromStr};

#[derive(Debug)]
pub enum LVal {
    Field(String),
    TmpVar(String),
    InstrPtr,
    Deref(Box<RVal>),
    Index(Box<RVal>, Box<RVal>),
}

impl fmt::Display for LVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LVal::Field(field) => write!(f, "self.{field}"),
            LVal::TmpVar(name) => write!(f, ".{name}"),
            LVal::InstrPtr => write!(f, "self.instr_ptr"),
            LVal::Deref(inner) => write!(f, "*{inner}"),
            LVal::Index(base, offset) => write!(f, "{base}[{offset}]"),
        }
    }
}

impl LVal {
    pub fn parser() -> impl Parser<char, Self, Error = Simple<char>> {
        let p_field = text::ident().map(LVal::Field);

        let p_tmp_var = just('.').ignore_then(text::ident()).map(LVal::TmpVar);

        let p_instr_ptr = just("instr_ptr").or(just("ip")).map(|_| LVal::InstrPtr);

        let p_deref = just('*')
            .ignore_then(RVal::parser())
            .map(Box::new)
            .map(LVal::Deref);

        let p_index = RVal::parser()
            .then(RVal::parser().delimited_by(just('['), just(']')))
            .map(|(base, offset)| LVal::Index(Box::new(base), Box::new(offset)));

        choice((p_field, p_tmp_var, p_instr_ptr, p_deref, p_index))
    }
}

impl FromStr for LVal {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self::parser().parse(s)?)
    }
}

pub struct LValFmt<'a> {
    pub(crate) lval: &'a LVal,
    pub(crate) mem: &'a Mem,
}

impl LVal {
    pub fn display<'a>(&'a self, mem: &'a Mem) -> LValFmt<'a> {
        LValFmt { lval: self, mem }
    }
}

impl std::fmt::Display for LValFmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.lval {
            LVal::Field(field) => write!(f, "self.{field}"),
            LVal::TmpVar(name) => write!(f, ".{name}"),
            LVal::InstrPtr => write!(f, "InstrPtr"),
            LVal::Deref(rval) => write!(f, "*{}", rval.display(self.mem)),
            LVal::Index(base, offset) => write!(
                f,
                "{}[{}]",
                base.display(self.mem),
                offset.display(self.mem)
            ),
        }
    }
}
