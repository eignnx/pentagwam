use std::fmt;

use serde::{Deserialize, Serialize};

use crate::human_powered_vm::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slice<T> {
    pub region: Region,
    pub start: T,
    pub len: T,
}

impl Slice<Option<i64>> {
    pub fn normalized_from(
        base: Slice<usize>,
        start: Option<i64>,
        len: Option<i64>,
    ) -> Result<Slice<usize>> {
        let slice = Slice {
            region: base.region,
            start,
            len,
        };
        slice.normalize(base)
    }

    /// Example:
    /// ```text
    /// [-4,-3,-2,-1,0,1,2,3,4]
    ///          a = ^
    /// (where X[INDEX;LENGTH] is a slice)
    /// a[0;3]   => [ 0, 1, 2] => a[4;3]
    /// a[0;-3]  => [-3,-2,-1] => a[-3;3]
    /// a[1;-3]  => [-2,-1, 0] => a[-2;3]
    /// a[-1;3]  => [-1, 0, 1] => a[-1;3]
    /// a[-1:-3] => [-4,-3,-2] => a[-4;3]
    /// ```
    pub fn normalize(&self, base: Slice<usize>) -> Result<Slice<usize>> {
        debug_assert_eq!(self.region, base.region);
        let region = self.region;

        let abs_start_signed = self.start.unwrap_or(0) + base.start as i64;
        // let Ok(abs_start) = usize::try_from(abs_start_signed) else {
        //     return Err(Error::BelowBoundsSliceStart(abs_start_signed));
        // };
        let abs_start = match usize::try_from(abs_start_signed) {
            Ok(abs_start) => abs_start,
            Err(_) => {
                println!("?> {}", Error::BelowBoundsSliceStart(abs_start_signed));
                0
            }
        };

        let (new_start, len) = match self.len {
            // Example:
            //              |<-----base.len------->|
            //              [*****|****************]
            // base.start-->|     |                |
            // abs_start----|---->|                |
            // new_start----|---->|<----- len ---->|
            None => (abs_start, base.start + base.len - abs_start),
            // Example:
            //              |<-----base.len------->|
            //              [*****|*********|******]
            // base.start-->|     |         |
            // abs_start----|---->|         |
            // new_start----|---->|<- len ->|
            Some(len) if len >= 0 => (abs_start, len as usize),
            // Example:
            //               |<----base.len----->|
            //               [**|*********|******]
            // base.start--->|  |         |
            // abs_start-----|--|-------->|
            //               |  |<- len ->|
            // new_start-----|->|
            Some(len) => (
                abs_start
                    .checked_sub(len.unsigned_abs() as usize)
                    .unwrap_or_else(|| {
                        println!(
                            "?> {}",
                            Error::BelowBoundsSliceStart(abs_start as i64 - len.abs())
                        );
                        0
                    }),
                len.unsigned_abs() as usize,
            ),
        };

        Ok(Slice {
            region,
            start: new_start,
            len,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Region {
    Mem,
    Code,
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Region::Mem => write!(f, "<heap-segment>"),
            Region::Code => write!(f, "<code-segment>"),
        }
    }
}
