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
    /// Example:
    /// ```text
    /// a = [0,1,2,3,4,5,6,7,8,9]
    /// (where X[INDEX;LENGTH] is a slice)
    /// a[4;3]   => [4,5,6] => a[4;3]
    /// a[4;-3]  => [2,3,4] => a[2;3]
    /// a[-4;3]  => [6,7,8] => a[6;3]
    /// a[-4:-3] => [4,5,6] => a[4;3]
    /// ```
    pub fn normalize(&self, base: Slice<usize>) -> Result<Slice<usize>> {
        debug_assert_eq!(self.region, base.region);
        let region = self.region;
        let start = self.start.unwrap_or(base.start as i64);
        let start = if start < 0 {
            (base.len as i64 + start) as usize
        } else {
            start as usize
        };

        let Some(rem_len_after_start) = base.len.checked_sub(start) else {
            return Err(Error::BadSliceBounds {
                base_len: base.len as i64,
                slice_start: start as i64,
                slice_len: self.len.unwrap_or(base.len as i64),
            });
        };

        // let len =
        todo!()
        // Slice {
        //     region: self.region,
        //     start: {
        //         let start = self.start.unwrap_or(0);
        //         if start < 0 {
        //             Some((base_len as i64 + start) as usize)
        //         } else {
        //             self.start.map(|i| i as usize)
        //         }
        //     },
        //     len: {
        //         let len = self.len.unwrap_or(base_len as i64);
        //         if len < 0 {
        //             Some((base_len as i64 + len) as usize)
        //         } else {
        //             self.len.map(|i| i as usize)
        //         }
        //     },
        // }
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
