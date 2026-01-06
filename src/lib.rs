#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::{
    cmp::{max, min},
    fmt::Debug,
    ops::Range,
};

pub mod core_ops;
mod heapless_ops;
pub mod helpers;

#[cfg(feature = "alloc")]
mod alloc_ops;

pub trait VecOps<T: RangeInfo> {
    fn push(&mut self, item: T) -> Result<(), RangeError<T>>;
    fn as_slice(&self) -> &[T];
    fn drain<R>(&mut self, range: R) -> impl Iterator<Item = T>
    where
        R: core::ops::RangeBounds<usize>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn remove(&mut self, index: usize) -> T;
    fn insert(&mut self, index: usize, item: T) -> Result<(), RangeError<T>>;
    fn clear(&mut self);
}

pub trait RangeSetBaseOps<T: RangeInfo>: VecOps<T> {
    fn merge_add_with_temp(
        &mut self,
        new_info: T,
        temp: &mut impl VecOps<T>,
    ) -> Result<(), RangeError<T>> {
        temp.clear();
        if !core_ops::validate_range(&new_info) {
            return Ok(());
        }

        // 检查冲突
        core_ops::check_conflicts(self.as_slice(), &new_info)?;

        for elem in self.drain(..) {
            if !helpers::ranges_overlap(&elem.range(), &new_info.range()) {
                temp.push(elem)?;
                continue;
            }

            if elem.kind() == new_info.kind() {
                temp.push(elem)?;
                continue;
            }

            let split_parts = helpers::split_range(&elem, &new_info.range());
            for part in split_parts.iter().flatten() {
                temp.push(part.clone())?;
            }
        }

        // 将处理后的结果复制回原数组（正序）
        for elem in temp.as_slice() {
            self.push(elem.clone())?;
        }

        // 插入新区间并合并
        if self.is_empty() {
            self.push(new_info).map_err(|_| RangeError::Capacity)?;
            return Ok(());
        }

        // 二分查找插入位置
        let insert_at = core_ops::find_insert_position(self.as_slice(), &new_info.range());

        let mut merged_range = new_info.range();
        let mut insert_at = insert_at;

        // 向左合并
        while insert_at > 0 {
            let left = &self.as_slice()[insert_at - 1];
            if left.range().end < merged_range.start || left.kind() != new_info.kind() {
                break;
            }
            merged_range.start = min(merged_range.start, left.range().start);
            merged_range.end = max(merged_range.end, left.range().end);
            self.remove(insert_at - 1);
            insert_at -= 1;
        }

        // 向右合并
        while insert_at < self.len() {
            let right = &self.as_slice()[insert_at];
            if right.range().start > merged_range.end || right.kind() != new_info.kind() {
                break;
            }
            merged_range.start = min(merged_range.start, right.range().start);
            merged_range.end = max(merged_range.end, right.range().end);
            self.remove(insert_at);
        }

        self.insert(insert_at, new_info.clone_with_range(merged_range))?;
        Ok(())
    }

    fn merge_remove_with_temp(
        &mut self,
        range: Range<T::Type>,
        temp: &mut impl VecOps<T>,
    ) -> Result<(), RangeError<T>> {
        temp.clear();
        if range.start >= range.end || self.is_empty() {
            return Ok(());
        }

        for elem in self.drain(..) {
            if !helpers::ranges_overlap(&elem.range(), &range) {
                temp.push(elem)?;
                continue;
            }

            let split_parts = helpers::split_range(&elem, &range);
            for part in split_parts.iter().flatten() {
                temp.push(part.clone())?;
            }
        }

        // 将处理后的结果复制回原数组（正序）
        for elem in temp.as_slice() {
            self.push(elem.clone())?;
        }

        Ok(())
    }
}

pub trait RangeSetOps<T: RangeInfo> {
    fn merge_add(&mut self, new_info: T, temp: &mut [u8]) -> Result<(), RangeError<T>>;

    fn merge_remove(&mut self, range: Range<T::Type>, temp: &mut [u8])
    -> Result<(), RangeError<T>>;

    fn merge_extend<I>(&mut self, ranges: I, temp: &mut [u8]) -> Result<(), RangeError<T>>
    where
        I: IntoIterator<Item = T>;

    fn merge_contains_point(&self, value: T::Type) -> bool;
}

/// RangeSet 操作 trait（alloc 版本），为带分配器的容器提供区间集合功能
/// 相比 RangeSetOps，不需要用户提供临时缓冲区
#[cfg(feature = "alloc")]
pub trait RangeSetAllocOps<T: RangeInfo> {
    /// 添加一个区间（会自动合并相邻区间）
    fn merge_add(&mut self, new_info: T) -> Result<(), RangeError<T>>;

    /// 删除一个区间
    fn merge_remove(&mut self, range: Range<T::Type>) -> Result<(), RangeError<T>>;

    /// 批量添加多个区间
    fn merge_extend<I>(&mut self, ranges: I) -> Result<(), RangeError<T>>
    where
        I: IntoIterator<Item = T>;

    /// 查询某个值是否落在任意一个区间中
    fn merge_contains_point(&self, value: T::Type) -> bool;
}

/// RangeSet 错误类型
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RangeError<T>
where
    T: RangeInfo,
{
    /// 容量不足错误
    #[error("RangeSet capacity exceeded")]
    Capacity,
    /// 区间冲突错误：尝试覆盖不可覆盖的区间
    #[error("Range conflict: new {new:?} conflicts with existing non-overwritable {existing:?}")]
    Conflict {
        /// 新添加的区间
        new: T,
        /// 已存在的冲突区间
        existing: T,
    },
}

pub trait RangeInfo: Debug + Clone + Sized + Default {
    type Kind: Debug + Eq + Clone;
    type Type: Ord + Copy;
    fn range(&self) -> Range<Self::Type>;
    fn kind(&self) -> Self::Kind;
    fn overwritable(&self) -> bool;
    fn clone_with_range(&self, range: Range<Self::Type>) -> Self;
}
