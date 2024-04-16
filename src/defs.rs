/// Generic process memory, no specific segment.
pub(crate) struct ProcMem;
/// For code.
pub(crate) struct CodeSeg;
pub(crate) struct HeapSeg;
pub(crate) struct StackSeg;
/// For static data.
pub(crate) struct DataSeg;

pub(crate) type Word = u32;
pub(crate) type Index = u32;

pub(crate) struct Offset<T> {
    pub(crate) offset: Index,
    pub(crate) _phantom: std::marker::PhantomData<T>,
}

impl<T> Offset<T> {
    pub(crate) const fn null() -> Offset<T> {
        Self {
            offset: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    pub(crate) const fn at(offset: Index) -> Self {
        Self {
            offset,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> std::ops::Add<Offset<T>> for Offset<T> {
    type Output = Offset<T>;

    fn add(self, rhs: Offset<T>) -> Self::Output {
        Self {
            offset: self.offset + rhs.offset,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> std::ops::Add<Index> for Offset<T> {
    type Output = Offset<T>;

    fn add(self, rhs: Index) -> Self::Output {
        Self {
            offset: self.offset + rhs,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> std::ops::Sub<Offset<T>> for Offset<T> {
    type Output = i32;

    fn sub(self, rhs: Offset<T>) -> Self::Output {
        (self.offset as i64 - rhs.offset as i64) as i32
    }
}
