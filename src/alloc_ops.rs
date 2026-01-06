use core::ops::Range;

use crate::{RangeError, RangeInfo, RangeSetAllocOps, RangeSetBaseOps, VecOps, core_ops};

impl<T: RangeInfo> VecOps<T> for alloc::vec::Vec<T> {
    fn push(&mut self, item: T) -> Result<(), RangeError<T>> {
        self.push(item);
        Ok(())
    }

    fn as_slice(&self) -> &[T] {
        self.as_slice()
    }

    fn drain<R>(&mut self, range: R) -> impl Iterator<Item = T>
    where
        R: core::ops::RangeBounds<usize>,
    {
        self.drain(range)
    }

    fn len(&self) -> usize {
        self.as_slice().len()
    }

    fn remove(&mut self, index: usize) -> T {
        self.remove(index)
    }

    fn insert(&mut self, index: usize, item: T) -> Result<(), RangeError<T>> {
        self.insert(index, item);
        Ok(())
    }

    fn clear(&mut self) {
        self.clear();
    }
}

impl<T: RangeInfo> RangeSetAllocOps<T> for alloc::vec::Vec<T> {
    fn merge_add(&mut self, new_info: T) -> Result<(), RangeError<T>> {
        let mut temp = alloc::vec::Vec::new();
        self.merge_add_with_temp(new_info, &mut temp)?;
        Ok(())
    }

    fn merge_remove(&mut self, range: Range<T::Type>) -> Result<(), RangeError<T>> {
        let mut temp = alloc::vec::Vec::new();
        self.merge_remove_with_temp(range, &mut temp)?;
        Ok(())
    }

    fn merge_extend<I>(&mut self, ranges: I) -> Result<(), RangeError<T>>
    where
        I: IntoIterator<Item = T>,
    {
        for info in ranges {
            self.merge_add(info)?;
        }

        Ok(())
    }

    fn merge_contains_point(&self, value: T::Type) -> bool {
        core_ops::contains_point(self.as_slice(), value)
    }
}

impl<T: RangeInfo> RangeSetBaseOps<T> for alloc::vec::Vec<T> {}
