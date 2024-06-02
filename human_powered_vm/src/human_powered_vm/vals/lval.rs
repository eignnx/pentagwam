use super::rval::RVal;
use crate::human_powered_vm::error::{Error, Result};
use chumsky::prelude::*;
use pentagwam::mem::{DisplayViaMem, Mem};
use std::{fmt, str::FromStr};

#[derive(Debug)]
pub enum LVal {
    Field(String),
    TmpVar(String),
    Deref(Box<RVal>),
    Index(Box<RVal>, Box<RVal>),
}

impl LVal {
    pub fn parser() -> impl Parser<char, Self, Error = Simple<char>> {
        let p_field = text::ident().map(LVal::Field);

        let p_tmp_var = just('.').ignore_then(text::ident()).map(LVal::TmpVar);

        let p_index_or_deref = RVal::atomic_rval_parser(RVal::parser())
            .map(Box::new)
            .then_with(|rval| {
                let rval_cpy = rval.clone();
                choice((
                    just(".*").map(move |_| LVal::Deref(rval_cpy.clone())),
                    RVal::parser()
                        .delimited_by(just('['), just(']'))
                        .map(move |offset| LVal::Index(rval.clone(), Box::new(offset))),
                ))
            });

        choice((p_field, p_tmp_var, p_index_or_deref))
    }
}

impl FromStr for LVal {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self::parser().parse(s)?)
    }
}

impl DisplayViaMem for LVal {
    fn display_via_mem(&self, f: &mut fmt::Formatter<'_>, mem: &Mem) -> fmt::Result {
        match self {
            LVal::Field(field) => write!(f, "self.{field}"),
            LVal::TmpVar(name) => write!(f, ".{name}"),
            LVal::Deref(rval) => write!(f, "{}.*", mem.display(rval)),
            LVal::Index(base, offset) => {
                write!(f, "{}[{}]", mem.display(base), mem.display(offset))
            }
        }
    }
}
