use std::{fmt, str::FromStr};

use super::*;

#[derive(Debug, From, Clone, Serialize, Deserialize)]
pub enum Val {
    #[from]
    CellRef(CellRef),
    Usize(usize),
    I32(i32),
    Cell(Cell),
}

impl Default for Val {
    fn default() -> Self {
        Self::Cell(Cell::Nil)
    }
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Val::CellRef(cell_ref) => write!(f, "{cell_ref}"),
            Val::Usize(u) => write!(f, "{u}"),
            Val::I32(i) => write!(f, "{i:+}"),
            Val::Cell(cell) => write!(f, "{cell:?}"),
        }
    }
}

impl Val {
    pub fn ty(&self) -> ValTy {
        match self {
            Val::CellRef(_) => ValTy::CellRef,
            Val::Usize(_) => ValTy::Usize,
            Val::I32(_) => ValTy::I32,
            Val::Cell(_) => ValTy::AnyCellVal,
        }
    }

    pub fn expect_cell_ref(&self) -> Result<CellRef> {
        match self {
            Val::CellRef(cell_ref) => Ok(*cell_ref),
            other => Err(Error::TypeError {
                expected: "CellRef".into(),
                received: other.ty(),
            }),
        }
    }

    pub fn expect_i32(&self) -> Result<i32> {
        match self {
            Val::I32(i) => Ok(*i),
            other => Err(Error::TypeError {
                expected: "i32".into(),
                received: other.ty(),
            }),
        }
    }

    pub fn expect_usize(&self) -> Result<usize> {
        match self {
            Val::Usize(u) => Ok(*u),
            other => Err(Error::TypeError {
                expected: "usize".into(),
                received: other.ty(),
            }),
        }
    }

    pub fn expect_cell(&self) -> Result<Cell> {
        match self {
            Val::Cell(cell) => Ok(*cell),
            other => Err(Error::TypeError {
                expected: "Cell".into(),
                received: other.ty(),
            }),
        }
    }
}

#[derive(Debug, From, Clone)]
pub(crate) enum RVal {
    Deref(Box<RVal>),
    #[from]
    CellRef(CellRef),
    Usize(usize),
    I32(i32),
    Field(String),
    InstrPtr,
    Cell(CellVal<RVal>),
}

/// Different from [`Cell`](pentagwam::cell::Cell) because it needs to be able
/// to compute subexpressions, and you don't want to have to deal with interned
/// [`Sym`](pentagwam::defs::Sym) or [`Functor`](pentagwam::cell::Functor)
/// indices.
#[derive(Debug, Clone)]
pub enum CellVal<T> {
    Ref(Box<T>),
    Rcd(Box<T>),
    Int(Box<T>),
    Sym(String),
    Sig { fname: String, arity: u8 },
    Lst(Box<T>),
    Nil,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValTy {
    CellRef,
    AnyCellVal,
    Usize,
    I32,
    Symbol,
    Functor,
    TypeOf(String),
}
impl ValTy {
    pub fn default_val(&self) -> Val {
        match self {
            ValTy::CellRef => Val::CellRef(CellRef::new(0)),
            ValTy::AnyCellVal => Val::Cell(Cell::Nil),
            ValTy::Usize => Val::Usize(0),
            ValTy::I32 => Val::I32(0),
            ValTy::Symbol => Val::Cell(Cell::Sym(Sym::new(0))),
            ValTy::Functor => Val::Cell(Cell::Sig(Functor {
                sym: Sym::new(0),
                arity: 0,
            })),
            ValTy::TypeOf(_) => panic!("Can't create default value for `TypeOf(..)`"),
        }
    }
}

impl fmt::Display for ValTy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValTy::CellRef => write!(f, "CellRef"),
            ValTy::AnyCellVal => write!(f, "AnyCellVal"),
            ValTy::Usize => write!(f, "Usize"),
            ValTy::I32 => write!(f, "I32"),
            ValTy::Symbol => write!(f, "Symbol"),
            ValTy::Functor => write!(f, "Functor"),
            ValTy::TypeOf(field) => write!(f, "TypeOf({field})"),
        }
    }
}

impl FromStr for ValTy {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "CellRef" => Ok(ValTy::CellRef),
            "AnyCellVal" => Ok(ValTy::AnyCellVal),
            "Usize" => Ok(ValTy::Usize),
            "I32" => Ok(ValTy::I32),
            "Symbol" => Ok(ValTy::Symbol),
            "Functor" => Ok(ValTy::Functor),
            _ if s.starts_with("TypeOf(") && s.ends_with(')') => {
                let field_name = &s["TypeOf(".len()..s.len() - 1];
                Ok(ValTy::TypeOf(field_name.to_string()))
            }
            _ => Err(Error::ParseTypeError(s.to_string())),
        }
    }
}

impl RVal {
    pub(crate) fn ty(&self) -> ValTy {
        match self {
            RVal::Deref(_) => ValTy::AnyCellVal,
            RVal::CellRef(_) => ValTy::CellRef,
            RVal::Usize(_) => ValTy::Usize,
            RVal::I32(_) => ValTy::I32,
            RVal::Field(field) => ValTy::TypeOf(field.clone()),
            RVal::InstrPtr => ValTy::Usize,
            RVal::Cell(_) => ValTy::AnyCellVal,
        }
    }
}

impl<T> CellVal<T> {
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

impl Default for RVal {
    fn default() -> Self {
        Self::Usize(0)
    }
}

impl fmt::Display for RVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RVal::Deref(inner) => write!(f, "*{inner}"),
            RVal::CellRef(cell_ref) => write!(f, "{cell_ref}"),
            RVal::Usize(u) => write!(f, "{u}"),
            RVal::I32(i) => write!(f, "{i:+}"),
            RVal::Field(field) => write!(f, "self.{field}"),
            RVal::InstrPtr => write!(f, "self.instr_ptr"),
            RVal::Cell(cell) => write!(f, "{cell:?}"),
        }
    }
}

impl FromStr for RVal {
    type Err = Error;

    fn from_str(rval: &str) -> Result<RVal> {
        match rval {
            _ if rval.starts_with('*') => {
                let inner = &rval[1..];
                let rval: RVal = inner.parse()?;
                Ok(RVal::Deref(Box::new(rval)))
            }
            _ if rval.starts_with('@') => {
                let u = rval[1..].parse::<usize>()?;
                Ok(RVal::CellRef(CellRef::new(u)))
            }
            _ if rval.starts_with(char::is_numeric) => {
                let u = rval.parse::<usize>()?;
                Ok(RVal::Usize(u))
            }
            _ if rval.starts_with(['+', '-']) => {
                let i = rval.parse::<i32>()?;
                Ok(RVal::I32(i))
            }
            "instruction_ptr" | "ip" => Ok(RVal::InstrPtr),
            _ if rval.parse::<CellVal<RVal>>().is_ok() => Ok(RVal::Cell(CellVal::from_str(rval)?)),
            _ if !rval.contains(char::is_whitespace) => Ok(RVal::Field(rval.to_string())),
            _ => Err(Error::UnknownRVal(rval.to_string())),
        }
    }
}

impl FromStr for CellVal<RVal> {
    type Err = Error;

    fn from_str(cell_val: &str) -> Result<Self> {
        match cell_val {
            _ if cell_val.starts_with("Rcd(") && cell_val.ends_with(')') => {
                let inner = &cell_val[4..cell_val.len() - 1];
                let cell_ref: RVal = inner.parse()?;
                Ok(CellVal::Rcd(Box::new(cell_ref)))
            }
            _ if cell_val.starts_with("Ref(") && cell_val.ends_with(')') => {
                let inner = &cell_val[4..cell_val.len() - 1];
                let cell_ref: RVal = inner.parse()?;
                Ok(CellVal::Ref(Box::new(cell_ref)))
            }
            _ if cell_val.starts_with("Int(") && cell_val.ends_with(')') => {
                let inner = &cell_val[4..cell_val.len() - 1];
                let int: RVal = inner.parse()?;
                Ok(CellVal::Int(Box::new(int)))
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
                Ok(CellVal::Lst(Box::new(cell_ref)))
            }
            "Nil" => Ok(CellVal::Nil),
            _ => Err(Error::UnknownRVal(cell_val.to_string())),
        }
    }
}

#[derive(Debug)]
pub(crate) enum LVal {
    Field(String),
    InstrPtr,
    CellRef(CellRef),
    Deref(Box<RVal>),
}

impl fmt::Display for LVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LVal::Field(field) => write!(f, "self.{field}"),
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
            _ if lval.starts_with(char::is_alphabetic) => Ok(LVal::Field(lval.to_string())),
            _ => Err(Error::UnknownLVal(lval.to_string())),
        }
    }
}
