use std::str::FromStr;

use super::{valty::ValTy, *};

/// Different from [`Cell`](pentagwam::cell::Cell) because it needs to be able
/// to compute subexpressions, and you don't want to have to deal with interned
/// [`Sym`](pentagwam::defs::Sym) or [`Functor`](pentagwam::cell::Functor)
/// indices.
#[derive(Debug, Clone)]
pub enum CellVal {
    Ref(RVal),
    Rcd(RVal),
    Int(RVal),
    Sym(String),
    Sig { fname: String, arity: u8 },
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
            CellVal::Sig { .. } => Some(ValTy::Functor),
            CellVal::Lst(_) => Some(ValTy::CellRef),
            CellVal::Nil => None,
        }
    }
}

impl FromStr for CellVal {
    type Err = Error;

    fn from_str(cell_val: &str) -> Result<Self> {
        match cell_val {
            _ if cell_val.starts_with("Rcd(") && cell_val.ends_with(')') => {
                let inner = &cell_val[4..cell_val.len() - 1];
                let cell_ref: RVal = inner.parse()?;
                Ok(CellVal::Rcd(cell_ref))
            }
            _ if cell_val.starts_with("Ref(") && cell_val.ends_with(')') => {
                let inner = &cell_val[4..cell_val.len() - 1];
                let cell_ref: RVal = inner.parse()?;
                Ok(CellVal::Ref(cell_ref))
            }
            _ if cell_val.starts_with("Int(") && cell_val.ends_with(')') => {
                let inner = &cell_val[4..cell_val.len() - 1];
                let int: RVal = inner.parse()?;
                Ok(CellVal::Int(int))
            }
            _ if cell_val.starts_with("Sym('") && cell_val.ends_with("')") => {
                let sym_text = &cell_val[5..cell_val.len() - 2];
                Ok(CellVal::Sym(sym_text.to_owned()))
            }
            _ if cell_val.starts_with("Sym(") && cell_val.ends_with(')') => {
                let sym_text = &cell_val[4..cell_val.len() - 1];
                Ok(CellVal::Sym(sym_text.to_owned()))
            }
            _ if cell_val.starts_with("Sig(") && cell_val.ends_with(')') => {
                let inner = &cell_val[4..cell_val.len() - 1];
                let (fname, arity) = inner
                    .split_once('/')
                    .ok_or(Error::CantParseFunctor(inner.to_owned()))?;
                let fname = if fname.starts_with('\'') && fname.ends_with('\'') {
                    &fname[1..fname.len() - 1]
                } else {
                    fname
                };
                Ok(CellVal::Sig {
                    fname: fname.to_owned(),
                    arity: arity.parse()?,
                })
            }
            _ if cell_val.starts_with("Lst(") && cell_val.ends_with(')') => {
                let inner = &cell_val[4..cell_val.len() - 1];
                let cell_ref: RVal = inner.parse()?;
                Ok(CellVal::Lst(cell_ref))
            }
            "Nil" => Ok(CellVal::Nil),
            _ => Err(Error::UnknownRVal(cell_val.to_string())),
        }
    }
}

pub struct FmtCellVal<'a> {
    cell: &'a CellVal,
}

impl CellVal {
    pub fn display(&self) -> FmtCellVal<'_> {
        FmtCellVal { cell: self }
    }
}

impl std::fmt::Display for FmtCellVal<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.cell {
            CellVal::Int(i) => write!(f, "{i:+}"),
            CellVal::Sig { fname, arity } => {
                if fname.contains(|c: char| !c.is_alphanumeric() && c != '_')
                    || !fname.starts_with(|c: char| c.is_alphabetic() || c == '_')
                {
                    write!(f, "Sig('{fname}'/{arity})")
                } else {
                    write!(f, "Sig({fname}/{arity})")
                }
            }
            CellVal::Sym(sym) => {
                if sym.contains(|c: char| !c.is_alphanumeric() && c != '_')
                    || !sym.starts_with(|c: char| c.is_alphabetic() || c == '_')
                {
                    write!(f, "Sym('{sym}')")
                } else {
                    write!(f, "Sym({sym})")
                }
            }
            CellVal::Ref(cell_ref) => write!(f, "Ref({cell_ref})"),
            CellVal::Rcd(cell_ref) => write!(f, "Rcd({cell_ref})"),
            CellVal::Lst(cell_ref) => write!(f, "Lst({cell_ref})"),
            CellVal::Nil => write!(f, "Nil"),
        }
    }
}
