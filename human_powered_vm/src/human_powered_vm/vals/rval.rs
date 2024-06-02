use crate::human_powered_vm::error::{Error, Result};

use crate::human_powered_vm::vals::cellval::CellVal;

use chumsky::prelude::*;
use derive_more::From;
use pentagwam::defs::CellRef;
use pentagwam::mem::{DisplayViaMem, Mem};
use std::{fmt, str::FromStr};

use super::valty::ValTy;

#[derive(Debug, From, Clone)]
pub enum RVal {
    AddressOf(Box<RVal>),
    Deref(Box<RVal>),
    Index(Box<RVal>, Box<RVal>),
    #[from]
    CellRef(CellRef),
    Usize(usize),
    I32(i32),
    Symbol(String),
    Field(String),
    TmpVar(String),
    Cell(Box<CellVal>),
}

impl Default for RVal {
    fn default() -> Self {
        Self::Usize(0)
    }
}

impl RVal {
    pub fn ty(&self) -> ValTy {
        match self {
            RVal::AddressOf(_) => ValTy::CellRef,
            RVal::Deref(_) => ValTy::Cell,
            RVal::Index(..) => ValTy::Cell,
            RVal::CellRef(_) => ValTy::CellRef,
            RVal::Usize(_) => ValTy::Usize,
            RVal::I32(_) => ValTy::I32,
            RVal::Symbol(_) => ValTy::Symbol,
            RVal::Field(field) => ValTy::TypeOf(field.clone()),
            RVal::TmpVar(name) => ValTy::TypeOf(name.clone()),
            RVal::Cell(_) => ValTy::Cell,
        }
    }

    pub fn atomic_rval_parser<'a>(
        rval: impl Parser<char, RVal, Error = Simple<char>> + 'a + Clone,
    ) -> impl Parser<char, Self, Error = Simple<char>> + 'a {
        let cell_lit = CellVal::parser(rval).map(Box::new).map(RVal::Cell);

        let cell_ref_lit = just("@")
            .ignore_then(text::digits(10))
            .try_map(|s: String, span| s.parse::<usize>().map_err(|e| Simple::custom(span, e)))
            .map(|u| RVal::CellRef(CellRef::new(u)));

        let usize_lit = text::digits(10)
            .try_map(|s: String, span| s.parse::<usize>().map_err(|e| Simple::custom(span, e)))
            .map(RVal::Usize);

        let i32_lit = one_of(['-', '+'])
            .then_with(|sign| {
                let sign = if sign == '-' { -1 } else { 1 };
                text::digits(10).try_map(move |s: String, span| {
                    s.parse::<i32>()
                        .map(move |x| x * sign)
                        .map_err(|e| Simple::custom(span, e))
                })
            })
            .map(RVal::I32);

        let sym_lit = just(":")
            .ignore_then(choice((
                just('\'')
                    .ignore_then(filter(|c| *c != '\'').repeated())
                    .then_ignore(just('\''))
                    .collect(),
                text::ident::<_, Simple<char>>(),
            )))
            .map(String::from)
            .map(RVal::Symbol);

        let tmp_var = just(".").ignore_then(text::ident()).map(RVal::TmpVar);

        let field = text::ident().map(RVal::Field);

        choice((
            cell_lit,
            cell_ref_lit,
            usize_lit,
            i32_lit,
            sym_lit,
            tmp_var,
            field,
        ))
    }

    pub fn parser() -> impl Parser<char, Self, Error = Simple<char>> + Clone {
        recursive::<_, _, _, _, Simple<char>>(|rval| {
            enum PostfixOp {
                Index(RVal),
                Deref,
                AddressOf,
            }

            Self::atomic_rval_parser(rval.clone())
                .then(
                    choice((
                        rval.clone()
                            .delimited_by(just("["), just("]"))
                            .map(PostfixOp::Index),
                        just(".*").map(|_| PostfixOp::Deref),
                        just(".&").map(|_| PostfixOp::AddressOf),
                    ))
                    .repeated(),
                )
                .foldl(|a, b| match b {
                    PostfixOp::Deref => RVal::Deref(Box::new(a)),
                    PostfixOp::Index(index) => RVal::Index(Box::new(a), Box::new(index)),
                    PostfixOp::AddressOf => RVal::AddressOf(Box::new(a)),
                })
                .boxed()
        })
    }
}

impl FromStr for RVal {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self::parser().parse(s)?)
    }
}

impl DisplayViaMem for RVal {
    fn display_via_mem(&self, f: &mut fmt::Formatter<'_>, mem: &Mem) -> fmt::Result {
        match self {
            RVal::AddressOf(inner) => write!(f, "{}.&", mem.display(inner)),
            RVal::Deref(inner) => write!(f, "{}.*", mem.display(inner)),
            RVal::Index(base, offset) => {
                write!(f, "{}[{}]", mem.display(base), mem.display(offset))
            }
            RVal::CellRef(r) => write!(f, "{r}"),
            RVal::Usize(u) => write!(f, "{u}"),
            RVal::I32(i) => write!(f, "{i:+}"),
            RVal::Symbol(s) => {
                if s.contains(|c: char| !c.is_alphanumeric() && c != '_')
                    || !s.starts_with(|c: char| c.is_alphabetic() || c == '_')
                {
                    write!(f, ":'{s}'")
                } else {
                    write!(f, ":{s}")
                }
            }
            RVal::Field(field) => write!(f, "self.{field}"),
            RVal::TmpVar(name) => write!(f, ".{name}"),
            RVal::Cell(cell) => write!(f, "{}", mem.display(cell)),
        }
    }
}
