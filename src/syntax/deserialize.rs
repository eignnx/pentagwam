use crate::{cell::Cell, defs::CellRef, mem::Mem};

use super::Term;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    ASigIsNotAValue(CellRef),
    ExpectedRcdToPointToSig(CellRef),
    BadCellRead(CellRef),
}

impl Term {
    pub fn deserialize(root: CellRef, mem: &Mem) -> Result<Self, Error> {
        match mem.try_cell_read(root).ok_or(Error::BadCellRead(root))? {
            Cell::Nil => Ok(Term::Nil),
            Cell::Ref(r1) => match mem.try_cell_read(r1).ok_or(Error::BadCellRead(r1))? {
                Cell::Ref(r2) if r1 == r2 => {
                    let name = mem.human_readable_var_name(r1).to_string();
                    if name.starts_with('_') {
                        Ok(Term::Var(None))
                    } else {
                        Ok(Term::Var(Some(name)))
                    }
                }
                _ => Term::deserialize(r1, mem),
            },
            Cell::Rcd(r) => {
                let Cell::Sig(f) = mem.try_cell_read(r).ok_or(Error::BadCellRead(r))? else {
                    return Err(Error::ExpectedRcdToPointToSig(r));
                };
                let sym = f.sym.resolve(mem).to_owned();
                let mut args = vec![];
                let arg_start = (r + 1).usize();
                let arg_end = arg_start + f.arity as usize;
                for arg_root in arg_start..arg_end {
                    args.push(Term::deserialize(arg_root.into(), mem)?);
                }
                Ok(Term::Record(sym, args))
            }
            Cell::Int(i) => Ok(Term::Int(i)),
            Cell::Sym(s) => Ok(Term::Sym(s.resolve(mem).to_owned())),
            Cell::Sig(_) => Err(Error::ASigIsNotAValue(root)),
            Cell::Lst(r) => {
                let car = Term::deserialize(r + 0, mem)?;
                let cdr = Term::deserialize(r + 1, mem)?;
                Ok(Term::Cons(Box::new(car), Box::new(cdr)))
            }
        }
    }
}
