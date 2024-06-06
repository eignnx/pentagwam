use std::fmt;

use pentagwam::mem::{DisplayViaMem, Mem};
use serde::{Deserialize, Serialize};

// use crate::human_powered_vm::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slice<I> {
    pub idx: Idx<I>,
    pub len: Len<I>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Idx<I> {
    /// Syntax: `lo`
    Lo,
    /// Syntax: `hi`
    Hi,
    Int(I),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Len<I> {
    /// Syntax: `++`
    PosInf,
    /// Syntax: `--`
    NegInf,
    Int(I),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Region {
    Mem,
    Code,
}

impl<I> Slice<I> {
    pub fn map_int<O, E>(&self, f: impl Fn(&I) -> Result<O, E>) -> Result<Slice<O>, E> {
        Ok(Slice {
            idx: self.idx.map_int(&f)?,
            len: self.len.map_int(&f)?,
        })
    }
}

impl<I> Idx<I> {
    pub fn map_int<O, E>(&self, f: impl Fn(&I) -> Result<O, E>) -> Result<Idx<O>, E> {
        Ok(match self {
            Idx::Lo => Idx::Lo,
            Idx::Hi => Idx::Hi,
            Idx::Int(i) => Idx::Int(f(i)?),
        })
    }
}

impl<I> Len<I> {
    pub fn map_int<O, E>(&self, f: impl Fn(&I) -> Result<O, E>) -> Result<Len<O>, E> {
        Ok(match self {
            Len::PosInf => Len::PosInf,
            Len::NegInf => Len::NegInf,
            Len::Int(i) => Len::Int(f(i)?),
        })
    }
}

// impl Slice<Option<i64>> {
//     pub fn normalized_from(
//         base: Slice<usize>,
//         start: Option<i64>,
//         len: Option<i64>,
//     ) -> Result<Slice<usize>> {
//         let slice = Slice {
//             region: base.region,
//             start,
//             len,
//         };
//         slice.normalize(base)
//     }

//     /// Example:
//     /// ```text
//     /// [-4,-3,-2,-1,0,1,2,3,4]
//     ///          a = ^
//     /// (where X[INDEX;LENGTH] is a slice)
//     /// a[0;3]   => [ 0, 1, 2] => a[4;3]
//     /// a[0;-3]  => [-3,-2,-1] => a[-3;3]
//     /// a[1;-3]  => [-2,-1, 0] => a[-2;3]
//     /// a[-1;3]  => [-1, 0, 1] => a[-1;3]
//     /// a[-1:-3] => [-4,-3,-2] => a[-4;3]
//     /// ```
//     pub fn normalize(&self, base: Slice<usize>) -> Result<Slice<usize>> {
//         debug_assert_eq!(self.region, base.region);
//         let region = self.region;

//         let abs_start_signed = self.start.unwrap_or(0) + base.start as i64;
//         // let Ok(abs_start) = usize::try_from(abs_start_signed) else {
//         //     return Err(Error::BelowBoundsSliceStart(abs_start_signed));
//         // };
//         let abs_start = match usize::try_from(abs_start_signed) {
//             Ok(abs_start) => abs_start,
//             Err(_) => {
//                 println!("?> {}", Error::BelowBoundsSliceStart(abs_start_signed));
//                 0
//             }
//         };

//         let (new_start, len) = match self.len {
//             // Example:
//             //              |<-----base.len------->|
//             //              [*****|****************]
//             // base.start-->|     |                |
//             // abs_start----|---->|                |
//             // new_start----|---->|<----- len ---->|
//             None => (abs_start, base.start + base.len - abs_start),
//             // Example:
//             //              |<-----base.len------->|
//             //              [*****|*********|******]
//             // base.start-->|     |         |
//             // abs_start----|---->|         |
//             // new_start----|---->|<- len ->|
//             Some(len) if len >= 0 => (abs_start, len as usize),
//             // Example:
//             //               |<----base.len----->|
//             //               [**|*********|******]
//             // base.start--->|  |         |
//             // abs_start-----|--|-------->|
//             //               |  |<- len ->|
//             // new_start-----|->|
//             Some(len) => (
//                 abs_start
//                     .checked_sub(len.unsigned_abs() as usize)
//                     .unwrap_or_else(|| {
//                         println!(
//                             "?> {}",
//                             Error::BelowBoundsSliceStart(abs_start as i64 - len.abs())
//                         );
//                         0
//                     }),
//                 len.unsigned_abs() as usize,
//             ),
//         };

//         Ok(Slice {
//             region,
//             start: new_start,
//             len,
//         })
//     }
// }

impl<I: fmt::Display> fmt::Display for Idx<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Idx::Lo => write!(f, "lo"),
            Idx::Hi => write!(f, "hi"),
            Idx::Int(i) => write!(f, "{i}"),
        }
    }
}

impl<I: DisplayViaMem> DisplayViaMem for Idx<I> {
    fn display_via_mem(&self, f: &mut fmt::Formatter<'_>, mem: &Mem) -> fmt::Result {
        match self {
            Idx::Lo => write!(f, "lo"),
            Idx::Hi => write!(f, "hi"),
            Idx::Int(i) => write!(f, "{}", mem.display(i)),
        }
    }
}

impl<I: std::ops::Add<i64>> std::ops::Add<i64> for Idx<I> {
    type Output = Idx<I::Output>;

    fn add(self, rhs: i64) -> Self::Output {
        match self {
            Idx::Lo => Idx::Lo,
            Idx::Hi => Idx::Hi,
            Idx::Int(i) => Idx::Int(i + rhs),
        }
    }
}

impl<I: fmt::Display> fmt::Display for Len<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Len::PosInf => write!(f, "++"),
            Len::NegInf => write!(f, "--"),
            Len::Int(i) => write!(f, "{i}"),
        }
    }
}

impl<I: DisplayViaMem> DisplayViaMem for Len<I> {
    fn display_via_mem(&self, f: &mut fmt::Formatter<'_>, mem: &Mem) -> fmt::Result {
        match self {
            Len::PosInf => write!(f, "++"),
            Len::NegInf => write!(f, "--"),
            Len::Int(i) => write!(f, "{}", mem.display(i)),
        }
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Region::Mem => write!(f, "<heap-segment>"),
            Region::Code => write!(f, "<code-segment>"),
        }
    }
}
