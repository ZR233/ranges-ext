#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::{
    cmp::{max, min},
    fmt::Debug,
    mem,
    ops::Range,
    slice,
};

use tinyvec::SliceVec;

/// RangeSet 操作 trait，为容器类型提供区间集合功能
pub trait RangeSetOps<T: RangeInfo> {
    /// 添加一个区间（会自动合并相邻区间）
    fn merge_add(&mut self, new_info: T, temp_buffer: &mut [u8]) -> Result<(), RangeError<T>>;

    /// 删除一个区间
    fn merge_remove(
        &mut self,
        range: Range<T::Type>,
        temp_buffer: &mut [u8],
    ) -> Result<(), RangeError<T>>;

    /// 批量添加多个区间
    fn merge_extend<I>(&mut self, ranges: I, temp_buffer: &mut [u8]) -> Result<(), RangeError<T>>
    where
        I: IntoIterator<Item = T>;

    /// 查询某个值是否落在任意一个区间中
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

pub type RangeSetHeapless<T, const C: usize = 128> = heapless::Vec<T, C>;
#[cfg(feature = "alloc")]
pub type RangeSetAlloc<T> = alloc::vec::Vec<T>;

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

/// 辅助函数模块
mod helpers {
    use super::*;

    /// 检查两个区间是否有交集
    #[inline]
    pub fn ranges_overlap<T1: Ord + Copy>(r1: &Range<T1>, r2: &Range<T1>) -> bool {
        !(r1.end <= r2.start || r1.start >= r2.end)
    }

    /// 分割区间：将原区间按分割范围分割成不重叠的部分
    pub fn split_range<T: RangeInfo>(elem: &T, split_range: &Range<T::Type>) -> [Option<T>; 2] {
        let elem_range = elem.range();
        let has_left = elem_range.start < split_range.start;
        let has_right = elem_range.end > split_range.end;

        match (has_left, has_right) {
            (true, true) => {
                let left = elem_range.start..split_range.start;
                let right = split_range.end..elem_range.end;
                [
                    if left.start < left.end {
                        Some(elem.clone_with_range(left))
                    } else {
                        None
                    },
                    if right.start < right.end {
                        Some(elem.clone_with_range(right))
                    } else {
                        None
                    },
                ]
            }
            (true, false) => {
                let left = elem_range.start..min(elem_range.end, split_range.start);
                [
                    if left.start < left.end {
                        Some(elem.clone_with_range(left))
                    } else {
                        None
                    },
                    None,
                ]
            }
            (false, true) => {
                let right = max(elem_range.start, split_range.end)..elem_range.end;
                [
                    None,
                    if right.start < right.end {
                        Some(elem.clone_with_range(right))
                    } else {
                        None
                    },
                ]
            }
            (false, false) => [None, None],
        }
    }

    /// 将字节缓冲区转换为 T 类型的可变切片
    #[inline]
    pub fn bytes_to_slice_mut<T>(buffer: &mut [u8]) -> &mut [T] {
        let len = buffer.len() / mem::size_of::<T>();
        let ptr = buffer.as_mut_ptr() as *mut T;
        unsafe { slice::from_raw_parts_mut(ptr, len) }
    }
}

/// 为 heapless::Vec 实现 RangeSetOps
impl<T: RangeInfo, const N: usize> RangeSetOps<T> for heapless::Vec<T, N> {
    fn merge_add(&mut self, new_info: T, temp_buffer: &mut [u8]) -> Result<(), RangeError<T>> {
        if new_info.range().start >= new_info.range().end {
            return Ok(());
        }

        // 检查冲突
        for elem in self.iter() {
            if !helpers::ranges_overlap(&elem.range(), &new_info.range()) {
                continue;
            }

            if elem.kind() == new_info.kind() {
                continue;
            }

            if !elem.overwritable() {
                return Err(RangeError::Conflict {
                    new: new_info,
                    existing: elem.clone(),
                });
            }
        }

        // 使用临时内存处理
        let temp_slice = helpers::bytes_to_slice_mut(temp_buffer);
        let mut out = SliceVec::from_slice_len(temp_slice, 0);

        for elem in self.drain(..) {
            if !helpers::ranges_overlap(&elem.range(), &new_info.range()) {
                out.push(elem);
                continue;
            }

            if elem.kind() == new_info.kind() {
                out.push(elem);
                continue;
            }

            let split_parts = helpers::split_range(&elem, &new_info.range());
            for part in split_parts.iter().flatten() {
                out.push(part.clone());
            }
        }

        // 将处理后的结果复制回原数组（正序）
        let out_len = out.len();
        for i in 0..out_len {
            self.push(out[i].clone())
                .map_err(|_| RangeError::Capacity)?;
        }

        // 插入新区间并合并
        if self.is_empty() {
            self.push(new_info).map_err(|_| RangeError::Capacity)?;
            return Ok(());
        }

        // 二分查找插入位置
        let insert_at = self
            .binary_search_by(|e| {
                if e.range().start < new_info.range().start {
                    core::cmp::Ordering::Less
                } else {
                    core::cmp::Ordering::Greater
                }
            })
            .unwrap_or_else(|pos| pos);

        let mut merged_range = new_info.range();
        let mut insert_at = insert_at;

        // 向左合并
        while insert_at > 0 {
            let left = &self[insert_at - 1];
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
            let right = &self[insert_at];
            if right.range().start > merged_range.end || right.kind() != new_info.kind() {
                break;
            }
            merged_range.start = min(merged_range.start, right.range().start);
            merged_range.end = max(merged_range.end, right.range().end);
            self.remove(insert_at);
        }

        self.insert(insert_at, new_info.clone_with_range(merged_range))
            .map_err(|_| RangeError::Capacity)?;
        Ok(())
    }

    fn merge_remove(
        &mut self,
        range: Range<T::Type>,
        temp_buffer: &mut [u8],
    ) -> Result<(), RangeError<T>> {
        if range.start >= range.end || self.is_empty() {
            return Ok(());
        }

        let temp_slice = helpers::bytes_to_slice_mut(temp_buffer);
        let mut out = SliceVec::from_slice_len(temp_slice, 0);

        for elem in self.drain(..) {
            if !helpers::ranges_overlap(&elem.range(), &range) {
                out.push(elem);
                continue;
            }

            let split_parts = helpers::split_range(&elem, &range);
            for part in split_parts.iter().flatten() {
                out.push(part.clone());
            }
        }

        // 将处理后的结果复制回原数组（正序）
        let out_len = out.len();
        for i in 0..out_len {
            self.push(out[i].clone())
                .map_err(|_| RangeError::Capacity)?;
        }
        Ok(())
    }

    fn merge_extend<I>(&mut self, ranges: I, temp_buffer: &mut [u8]) -> Result<(), RangeError<T>>
    where
        I: IntoIterator<Item = T>,
    {
        for v in ranges {
            self.merge_add(v, temp_buffer)?;
        }
        Ok(())
    }

    fn merge_contains_point(&self, value: T::Type) -> bool {
        self.binary_search_by(|e| {
            if e.range().end <= value {
                core::cmp::Ordering::Less
            } else if e.range().start > value {
                core::cmp::Ordering::Greater
            } else {
                core::cmp::Ordering::Equal
            }
        })
        .is_ok()
    }
}

/// 为 alloc::vec::Vec 实现 RangeSetAllocOps
#[cfg(feature = "alloc")]
impl<T: RangeInfo> RangeSetAllocOps<T> for alloc::vec::Vec<T> {
    fn merge_add(&mut self, new_info: T) -> Result<(), RangeError<T>> {
        if new_info.range().start >= new_info.range().end {
            return Ok(());
        }

        // 检查冲突
        for elem in self.iter() {
            if !helpers::ranges_overlap(&elem.range(), &new_info.range()) {
                continue;
            }

            if elem.kind() == new_info.kind() {
                continue;
            }

            if !elem.overwritable() {
                return Err(RangeError::Conflict {
                    new: new_info,
                    existing: elem.clone(),
                });
            }
        }

        // 使用 Vec 作为临时存储
        let mut out = alloc::vec::Vec::new();

        for elem in self.drain(..) {
            if !helpers::ranges_overlap(&elem.range(), &new_info.range()) {
                out.push(elem);
                continue;
            }

            if elem.kind() == new_info.kind() {
                out.push(elem);
                continue;
            }

            let split_parts = helpers::split_range(&elem, &new_info.range());
            for part in split_parts.iter().flatten() {
                out.push(part.clone());
            }
        }

        // 将处理后的结果赋值回原数组
        *self = out;

        // 插入新区间并合并
        if self.is_empty() {
            self.push(new_info);
            return Ok(());
        }

        // 二分查找插入位置
        let insert_at = self
            .binary_search_by(|e| {
                if e.range().start < new_info.range().start {
                    core::cmp::Ordering::Less
                } else {
                    core::cmp::Ordering::Greater
                }
            })
            .unwrap_or_else(|pos| pos);

        let mut merged_range = new_info.range();
        let mut insert_at = insert_at;

        // 向左合并
        while insert_at > 0 {
            let left = &self[insert_at - 1];
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
            let right = &self[insert_at];
            if right.range().start > merged_range.end || right.kind() != new_info.kind() {
                break;
            }
            merged_range.start = min(merged_range.start, right.range().start);
            merged_range.end = max(merged_range.end, right.range().end);
            self.remove(insert_at);
        }

        self.insert(insert_at, new_info.clone_with_range(merged_range));
        Ok(())
    }

    fn merge_remove(&mut self, range: Range<T::Type>) -> Result<(), RangeError<T>> {
        if range.start >= range.end || self.is_empty() {
            return Ok(());
        }

        // 使用 Vec 作为临时存储
        let mut out = alloc::vec::Vec::new();

        for elem in self.drain(..) {
            if !helpers::ranges_overlap(&elem.range(), &range) {
                out.push(elem);
                continue;
            }

            let split_parts = helpers::split_range(&elem, &range);
            for part in split_parts.iter().flatten() {
                out.push(part.clone());
            }
        }

        // 将处理后的结果赋值回原数组
        *self = out;
        Ok(())
    }

    fn merge_extend<I>(&mut self, ranges: I) -> Result<(), RangeError<T>>
    where
        I: IntoIterator<Item = T>,
    {
        for v in ranges {
            self.merge_add(v)?;
        }
        Ok(())
    }

    fn merge_contains_point(&self, value: T::Type) -> bool {
        self.binary_search_by(|e| {
            if e.range().end <= value {
                core::cmp::Ordering::Less
            } else if e.range().start > value {
                core::cmp::Ordering::Greater
            } else {
                core::cmp::Ordering::Equal
            }
        })
        .is_ok()
    }
}
