use std::fmt;

use self::vals::ValTy;

use super::*;

#[derive(Debug, From)]
pub enum Error {
    UnknownRVal(String),
    UnknownLVal(String),
    #[from]
    ParseIntError(std::num::ParseIntError),
    /// Tried to assign an r-value of type `T` to an l-value of type `U`.
    AssignmentTypeError {
        expected: String,
        received: ValTy,
    },
    #[from]
    IoError(std::io::Error),
    BadSaveFileFormat,
    UndefinedField(String),
    OutOfBoundsMemRead(CellRef),
    OutOfBoundsMemWrite(CellRef),
    CantParseFunctor(String),
    TypeError {
        expected: String,
        received: ValTy,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownRVal(rval) => write!(f, "Unknown r-value `{rval}`."),
            Error::UnknownLVal(lval) => write!(f, "Unknown l-value `{lval}`."),
            Error::AssignmentTypeError { expected, received } => write!(
                f,
                "Assignment type error: Expected `{expected}`, but received `{received:?}`."
            ),
            Error::TypeError { expected, received } => write!(
                f,
                "Type error: Expected `{expected}`, but received `{received:?}`."
            ),
            Error::IoError(e) => write!(f, "I/O error: {e}"),
            Error::ParseIntError(e) => write!(f, "Parse int error: {e}"),
            Error::BadSaveFileFormat => write!(f, "Bad save file format."),
            Error::UndefinedField(field) => write!(f, "Undefined field `{field}`"),
            Error::OutOfBoundsMemRead(cell_ref) => {
                write!(f, "Out of bounds memory READ: {cell_ref}")
            }
            Error::OutOfBoundsMemWrite(cell_ref) => {
                write!(f, "Out of bounds memory WRITE: {cell_ref}")
            }
            Error::CantParseFunctor(text) => write!(
                f,
                "Can't parse functor (format -> SYMBOL/ARITY <-): `{text}`"
            ),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;