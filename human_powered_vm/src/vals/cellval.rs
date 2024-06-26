use core::fmt;

use self::rval::RVal;
use super::{valty::ValTy, *};
use chumsky::prelude::*;
use pentagwam::mem::{DisplayViaMem, Mem};

/// Different from [`Cell`](pentagwam::cell::Cell) because it needs to be able
/// to compute subexpressions, and you don't want to have to deal with interned
/// [`Sym`](pentagwam::defs::Sym) or [`Functor`](pentagwam::cell::Functor)
/// indices.
#[derive(Debug, Clone)]
pub enum CellVal {
    Ref(RVal),
    Rcd(RVal),
    Int(RVal),
    Sym(RVal),
    Sig(RVal),
    Lst(RVal),
    Nil,
}

impl CellVal {
    pub fn arg_ty(&self) -> Option<ValTy> {
        match self {
            CellVal::Ref(_) => Some(ValTy::CellRef),
            CellVal::Rcd(_) => Some(ValTy::CellRef),
            CellVal::Int(_) => Some(ValTy::I32),
            CellVal::Sym(_) => Some(ValTy::Symbol),
            CellVal::Sig(_) => Some(ValTy::Functor),
            CellVal::Lst(_) => Some(ValTy::CellRef),
            CellVal::Nil => None,
        }
    }

    pub fn parser<'a>(
        rval: impl Parser<char, RVal, Error = Simple<char>> + 'a + Clone,
    ) -> impl Parser<char, Self, Error = Simple<char>> + 'a + Clone {
        let p_ref = just("Ref")
            .ignore_then(rval.clone().delimited_by(just('('), just(')')))
            .map(CellVal::Ref);

        let p_rcd = just("Rcd")
            .ignore_then(rval.clone().delimited_by(just('('), just(')')))
            .map(CellVal::Rcd);

        let p_int = just("Int")
            .ignore_then(rval.clone().delimited_by(just('('), just(')')))
            .map(CellVal::Int);

        let p_sym = just("Sym")
            .ignore_then(rval.clone().delimited_by(just('('), just(')'))) // TODO: make symbol literal RVal
            .map(CellVal::Sym);

        let p_sig = just("Sig")
            .ignore_then(rval.clone().delimited_by(just('('), just(')')))
            .map(CellVal::Sig);

        let p_lst = just("Lst")
            .ignore_then(rval.clone().delimited_by(just('('), just(')')))
            .map(CellVal::Lst);

        let p_nil = just("Nil").map(|_| CellVal::Nil);

        choice((p_ref, p_rcd, p_int, p_sym, p_sig, p_lst, p_nil))
    }
}

impl DisplayViaMem for CellVal {
    fn display_via_mem(&self, f: &mut fmt::Formatter<'_>, mem: &Mem) -> fmt::Result {
        match self {
            CellVal::Int(i) => write!(f, "{}", mem.display(i)),
            CellVal::Sig(functor) => write!(f, "Sig({}", mem.display(functor)),
            CellVal::Sym(sym) => write!(f, "Sym({})", mem.display(sym)),
            CellVal::Ref(cell_ref) => write!(f, "Ref({})", mem.display(cell_ref)),
            CellVal::Rcd(cell_ref) => write!(f, "Rcd({})", mem.display(cell_ref)),
            CellVal::Lst(cell_ref) => write!(f, "Lst({})", mem.display(cell_ref)),
            CellVal::Nil => write!(f, "Nil"),
        }
    }
}
