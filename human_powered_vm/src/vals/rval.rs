use chumsky::prelude::*;
use derive_more::From;
use pentagwam::defs::CellRef;
use pentagwam::mem::{DisplayViaMem, Mem};
use std::{fmt, str::FromStr};

use super::valty::CellTy;
use super::{
    cellval::CellVal,
    slice::{self, Idx, Len, Slice},
    valty::ValTy,
};
use crate::human_powered_vm::error::{Error, Result};
use crate::human_powered_vm::HumanPoweredVm;

#[derive(Debug, From, Clone)]
pub enum RVal {
    AddressOf(Box<RVal>),
    Deref(Box<RVal>),
    Index(Box<RVal>, Box<Idx<RVal>>),
    IndexSlice(Box<RVal>, Box<Slice<RVal>>),
    #[from]
    CellRef(CellRef),
    Usize(usize),
    I32(i32),
    Symbol(String),
    Field(String),
    TmpVar(String),
    InstrParam(usize),
    Cell(Box<CellVal>),
    Functor(Box<RVal>, Box<RVal>),
}

impl Default for RVal {
    fn default() -> Self {
        Self::Usize(0)
    }
}

pub const SLICE_IDX_LEN_SEP: &str = ";";

impl RVal {
    pub fn ty(&self, hpvm: &HumanPoweredVm) -> Result<ValTy> {
        Ok(match self {
            RVal::AddressOf(_) => ValTy::CellRef,
            RVal::Cell(cell) => match cell.as_ref() {
                CellVal::Sig { .. } => ValTy::Cell(Some(CellTy::Sig)),
                CellVal::Int(_) => ValTy::Cell(Some(CellTy::Int)),
                CellVal::Sym(_) => ValTy::Cell(Some(CellTy::Sym)),
                CellVal::Ref(_) => ValTy::Cell(Some(CellTy::Ref)),
                CellVal::Rcd(_) => ValTy::Cell(Some(CellTy::Rcd)),
                CellVal::Lst(_) => ValTy::Cell(Some(CellTy::Lst)),
                CellVal::Nil => ValTy::Cell(Some(CellTy::Nil)),
            },
            RVal::Deref(_) => ValTy::Cell(None),
            RVal::Index(..) => ValTy::Cell(None),
            RVal::CellRef(_) => ValTy::CellRef,
            RVal::Field(field) => {
                hpvm.fields
                    .get(field)
                    .ok_or(Error::UndefinedField(field.clone()))?
                    .ty
            }
            RVal::I32(_) => ValTy::I32,
            RVal::Usize(_) => ValTy::Usize,
            RVal::IndexSlice(..) => ValTy::Slice,
            RVal::Symbol(_) => ValTy::Symbol,
            RVal::TmpVar(name) => {
                hpvm.tmp_vars
                    .get(name)
                    .ok_or(Error::UndefinedTmpVar(name.clone()))?
                    .ty
            }
            RVal::InstrParam(idx) => {
                let param = hpvm.instr_param(*idx)?;
                param.ty(hpvm)?
            }
            RVal::Functor(_, _) => ValTy::Functor,
        })
    }

    pub fn atomic_rval_parser<'a>(
        rval: impl Parser<char, RVal, Error = Simple<char>> + 'a + Clone,
    ) -> impl Parser<char, Self, Error = Simple<char>> + 'a {
        let cell_lit = CellVal::parser(rval.clone())
            .map(Box::new)
            .map(RVal::Cell)
            .labelled("cell literal");

        let cell_ref_lit = just("@")
            .ignore_then(text::digits(10))
            .try_map(|s: String, span| s.parse::<usize>().map_err(|e| Simple::custom(span, e)))
            .map(|u| RVal::CellRef(CellRef::new(u)))
            .labelled("cell ref literal");

        let usize_lit = text::digits(10)
            .try_map(|s: String, span| s.parse::<usize>().map_err(|e| Simple::custom(span, e)))
            .map(RVal::Usize)
            .labelled("usize literal");

        let i32_lit = one_of(['-', '+'])
            .then_with(|sign| {
                let sign = if sign == '-' { -1 } else { 1 };
                text::digits(10).try_map(move |s: String, span| {
                    s.parse::<i32>()
                        .map(move |x| x * sign)
                        .map_err(|e| Simple::custom(span, e))
                })
            })
            .map(RVal::I32)
            .labelled("i32 literal");

        let sym_lit = just(":")
            .ignore_then(choice((
                just('\'')
                    .ignore_then(filter(|c| *c != '\'').repeated())
                    .then_ignore(just('\''))
                    .collect(),
                text::ident::<_, Simple<char>>(),
            )))
            .map(String::from)
            .map(RVal::Symbol)
            .labelled("symbol literal");

        let tmp_var = just(".")
            .ignore_then(text::ident())
            .map(RVal::TmpVar)
            .labelled("temporary variable");

        let field = text::ident().map(RVal::Field).labelled("field name");

        let instr_param = just("$")
            .ignore_then(text::digits(10))
            .try_map(|s: String, span| s.parse::<usize>().map_err(|e| Simple::custom(span, e)))
            .map(RVal::InstrParam)
            .labelled("instruction parameter");

        choice((
            cell_lit,
            cell_ref_lit,
            usize_lit,
            i32_lit,
            sym_lit,
            tmp_var,
            field,
            instr_param,
        ))
    }

    pub fn parser() -> impl Parser<char, Self, Error = Simple<char>> + Clone {
        recursive::<_, _, _, _, Simple<char>>(|rval| {
            enum PostfixOp {
                Index(Idx<RVal>),
                IndexSlice(Slice<RVal>),
                Deref,
                AddressOf,
                Functor(Box<RVal>),
            }

            let idx_bound_p = choice((
                rval.clone().map(Idx::Int),
                just(slice::LO_TOK).map(|_| Idx::Lo),
                just(slice::HI_TOK).map(|_| Idx::Hi),
            ));

            let len_bound_p = choice((
                rval.clone().map(Len::Int),
                just(slice::POS_INF_TOK).map(|_| Len::PosInf),
                just(slice::NEG_INF_TOK).map(|_| Len::NegInf),
            ));

            let index_slice_p = idx_bound_p
                .clone()
                .then_ignore(just(SLICE_IDX_LEN_SEP))
                .then(len_bound_p)
                .map(|(idx, len)| PostfixOp::IndexSlice(Slice { idx, len }));

            let index_p = idx_bound_p.map(PostfixOp::Index);
            let deref_p = just(".*").map(|_| PostfixOp::Deref);
            let addr_of_p = just(".&").map(|_| PostfixOp::AddressOf);

            let functor_p = just("/")
                .ignore_then(rval.clone())
                .map(|arity| PostfixOp::Functor(Box::new(arity)))
                .labelled("functor literal");

            Self::atomic_rval_parser(rval.clone())
                .then(
                    choice((
                        choice((index_slice_p, index_p)).delimited_by(just("["), just("]")),
                        deref_p,
                        addr_of_p,
                        functor_p,
                    ))
                    .repeated(),
                )
                .foldl(|acc, new| match new {
                    PostfixOp::Index(index) => RVal::Index(Box::new(acc), Box::new(index)),
                    PostfixOp::IndexSlice(Slice { idx, len }) => {
                        RVal::IndexSlice(Box::new(acc), Box::new(Slice { idx, len }))
                    }
                    PostfixOp::Deref => RVal::Deref(Box::new(acc)),
                    PostfixOp::AddressOf => RVal::AddressOf(Box::new(acc)),
                    PostfixOp::Functor(arity) => RVal::Functor(Box::new(acc), arity),
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
            RVal::Index(base, idx) => {
                write!(f, "{}[{}]", mem.display(base), mem.display(idx))
            }
            RVal::IndexSlice(base, slice) => {
                let Slice { idx, len } = slice.as_ref();
                write!(
                    f,
                    "{}[{}{SLICE_IDX_LEN_SEP}{}]",
                    mem.display(base),
                    mem.display(idx),
                    mem.display(len)
                )
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
            RVal::Field(field) => write!(f, "{field}"),
            RVal::TmpVar(name) => write!(f, ".{name}"),
            RVal::InstrParam(idx) => write!(f, "${idx}"),
            RVal::Cell(cell) => write!(f, "{}", mem.display(cell)),
            RVal::Functor(sym, arity) => write!(f, "({}/{})", mem.display(sym), mem.display(arity)),
        }
    }
}
