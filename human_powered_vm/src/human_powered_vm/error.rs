use derive_more::From;
use std::fmt;

use crate::vals::{slice::Region, valty::ValTy};

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
    BadSaveFileFormat(String),
    UndefinedField(String),
    UndefinedTmpVar(String),
    OutOfBoundsMemRead(Region, usize),
    OutOfBoundsMemWrite(Region, usize),
    CantParseFunctor(String),
    TypeError {
        expected: String,
        received: ValTy,
        expr: String,
    },
    ParseTypeError(String),
    #[from]
    RonDeSpannedError(ron::de::SpannedError),
    #[from]
    ChumskyParseError(Vec<chumsky::error::Simple<char>>),
    BadAddressOfArgument {
        reason: &'static str,
        value: String,
    },
    UnsliceableValue(String),
    BadSliceBounds {
        base_len: i64,
        slice_start: i64,
        slice_len: i64,
    },
    BelowBoundsSliceStart(i64),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownRVal(rval) => write!(f, "Unknown r-value `{rval}`."),
            Error::UnknownLVal(lval) => write!(f, "Unknown l-value `{lval}`."),
            Error::AssignmentTypeError { expected, received } => write!(
                f,
                "Assignment type error: Could not assign value of type `{received}` to a location which holds `{expected}`s."
            ),
            Error::TypeError { expected, received, expr } => write!(
                f,
                "Type error: Expected `{expected}`, but received `{expr}: {received}`."
            ),
            Error::IoError(e) => write!(f, "I/O error: {e}"),
            Error::ParseIntError(e) => write!(f, "Parse int error: {e}"),
            Error::BadSaveFileFormat(line) => write!(f, "Bad save file format: {line}"),
            Error::UndefinedField(field) => write!(f, "Undefined field `{field}`"),
            Error::UndefinedTmpVar(name) => write!(f, "Undefined temporary variable `.{name}`"),
            Error::OutOfBoundsMemRead(region, cell_ref) => {
                write!(f, "Out of bounds memory READ: {region}[{cell_ref}]")
            }
            Error::OutOfBoundsMemWrite(region, cell_ref) => {
                write!(f, "Out of bounds memory WRITE: {region}[{cell_ref}]")
            }
            Error::CantParseFunctor(text) => write!(
                f,
                "Can't parse functor (format -> SYMBOL/ARITY <-): `{text}`"
            ),
            Error::ParseTypeError(text) => write!(f, "Can't parse type: `{text}`"),
            Error::RonDeSpannedError(e) => write!(f, "Error while parsing save file: {e}"),
            Error::ChumskyParseError(es) => {
                writeln!(f, "Parse error:")?;
                for e in es {
                    writeln!(f, "\t{e}")?;
                }
                Ok(())
            }
            Error::BadAddressOfArgument { reason, value } => {
                writeln!(f, "Bad address-of argument `{value}`: {reason}")
            }
            Error::UnsliceableValue(val) => {
                writeln!(f, "Can't slice value `{val}`. Only values which \
                             evaluate to a CellRef or a Usize (corresponding to \
                             the heap and code segments, respectively) or to a \
                             Slice can be sliced.")

            }
            Error::BadSliceBounds { base_len: old_len, slice_start: new_start, slice_len: new_len } => {
                writeln!(f, "Bad slice: `<…>[{new_start}..{new_len}]` produces \
                             a slice of length `{new_len}`, but the original \
                             slice was of length `{old_len}`, and you asked \
                             for the subslice to begin at index `{new_start}`. \
                             That leaves only `{}` elements from the original \
                             slice available to subslice.", old_len - new_start)
            }
            Error::BelowBoundsSliceStart(below) => writeln!(
                f,
                "Attempt to index at index less than absolute address 0: {below}",
            ),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
