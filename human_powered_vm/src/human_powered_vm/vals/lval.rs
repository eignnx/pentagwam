use super::*;
use std::{fmt, str::FromStr};

#[derive(Debug)]
pub(crate) enum LVal {
    Field(String),
    TmpVar(String),
    InstrPtr,
    CellRef(CellRef),
    Deref(Box<RVal>),
}

impl fmt::Display for LVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LVal::Field(field) => write!(f, "self.{field}"),
            LVal::TmpVar(name) => write!(f, ".{name}"),
            LVal::InstrPtr => write!(f, "self.instr_ptr"),
            LVal::CellRef(cell_ref) => write!(f, "{cell_ref}"),
            LVal::Deref(inner) => write!(f, "*{inner}"),
        }
    }
}

impl FromStr for LVal {
    type Err = Error;

    fn from_str(lval: &str) -> Result<LVal> {
        match lval {
            "instr_ptr" | "ip" => Ok(LVal::InstrPtr),
            _ if lval.starts_with('*') => {
                let inner = &lval[1..];
                let inner: RVal = inner.parse()?;
                Ok(LVal::Deref(Box::new(inner)))
            }
            _ if lval.starts_with('@') => {
                let u = lval[1..].parse::<usize>()?;
                Ok(LVal::CellRef(CellRef::new(u)))
            }
            _ if lval.starts_with('.') => {
                let name = &lval[1..];
                Ok(LVal::TmpVar(name.to_string()))
            }
            _ if lval.starts_with(char::is_alphabetic) => Ok(LVal::Field(lval.to_string())),
            _ => Err(Error::UnknownLVal(lval.to_string())),
        }
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
            LVal::CellRef(cell_ref) => write!(f, "{cell_ref}"),
            LVal::Field(field) => write!(f, "self.{field}"),
            LVal::TmpVar(name) => write!(f, ".{name}"),
            LVal::InstrPtr => write!(f, "InstrPtr"),
            LVal::Deref(rval) => write!(f, "*{}", rval.display(self.mem)),
        }
    }
}
