use core::ops::Range;

use tinyvec::SliceVec;

use crate::{
    RangeError, RangeInfo, RangeSetBaseOps, RangeSetOps, VecOps, core_ops,
    helpers::bytes_to_slice_mut,
};

impl<T: RangeInfo, const N: usize> RangeSetOps<T> for heapless::Vec<T, N> {
    fn merge_add(&mut self, new_info: T, temp: &mut [u8]) -> Result<(), RangeError<T>> {
        let temp_buff = bytes_to_slice_mut::<T>(temp);
        let mut temp = SliceVec::from_slice_len(temp_buff, 0);
        self.merge_add_with_temp(new_info, &mut temp)?;
        Ok(())
    }

    fn merge_remove(
        &mut self,
        range: Range<T::Type>,
        temp: &mut [u8],
    ) -> Result<(), RangeError<T>> {
        let temp_buff = bytes_to_slice_mut::<T>(temp);
        let mut temp = SliceVec::from_slice_len(temp_buff, 0);
        self.merge_remove_with_temp(range, &mut temp)?;
        Ok(())
    }

    fn merge_extend<I>(&mut self, ranges: I, temp: &mut [u8]) -> Result<(), RangeError<T>>
    where
        I: IntoIterator<Item = T>,
    {
        for info in ranges {
            self.merge_add(info, temp)?;
        }

        Ok(())
    }

    fn merge_contains_point(&self, value: T::Type) -> bool {
        core_ops::contains_point(self.as_slice(), value)
    }
}

impl<T: RangeInfo, const N: usize> VecOps<T> for heapless::Vec<T, N> {
    fn push(&mut self, item: T) -> Result<(), RangeError<T>> {
        self.push(item).map_err(|_| RangeError::Capacity)
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
        self.insert(index, item).map_err(|_| RangeError::Capacity)
    }

    fn clear(&mut self) {
        self.clear();
    }
}

impl<T: RangeInfo, const N: usize> RangeSetBaseOps<T> for heapless::Vec<T, N> {}

impl<T: RangeInfo> VecOps<T> for SliceVec<'_, T> {
    fn push(&mut self, item: T) -> Result<(), RangeError<T>> {
        if self.len() >= self.capacity() {
            return Err(RangeError::Capacity);
        }
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
        if index > self.len() || self.len() >= self.capacity() {
            return Err(RangeError::Capacity);
        }

        self.insert(index, item);
        Ok(())
    }

    fn clear(&mut self) {
        self.clear();
    }
}
