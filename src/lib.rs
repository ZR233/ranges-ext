#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::{
    cmp::{max, min},
    fmt::Debug,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut, Range, RangeBounds},
    slice,
};

use tinyvec::SliceVec;

pub type RangeSetHeapless<T, const C: usize = 128> = RangeSet<T, heapless::Vec<T, C>>;
#[cfg(feature = "alloc")]
pub type RangeSetAlloc<T> = RangeSet<T, alloc::vec::Vec<T>>;

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

pub trait VecOp<T>: Default + Deref<Target = [T]> + DerefMut {
    fn push_back(&mut self, item: T) -> Result<(), T>;
    fn clear(&mut self);
    fn insert(&mut self, index: usize, item: T) -> Result<(), T>;
    fn remove(&mut self, index: usize) -> T;
    fn drain<R>(&mut self, range: R) -> impl Iterator<Item = T>
    where
        R: RangeBounds<usize>;
}

/// 一个「区间集合」数据结构：维护一组**有序、互不重叠**的半开区间 `[start, end)`。
///
/// - 插入时会把重叠/相邻且 kind 相等的区间自动合并。
/// - 删除一个区间时，会从集合里移除交集；必要时把已有区间拆成左右两段。
///
/// 约定：空区间（`start >= end`）会被忽略。
#[derive(Clone, Debug)]
pub struct RangeSet<T, V>
where
    T: RangeInfo,
    V: VecOp<T>,
{
    elements: V,
    _marker: PhantomData<T>,
}

impl<T, V> Default for RangeSet<T, V>
where
    T: RangeInfo,
    V: VecOp<T>,
{
    fn default() -> Self {
        Self::new(V::default())
    }
}

impl<T, V> RangeSet<T, V>
where
    T: RangeInfo,
    V: VecOp<T>,
{
    /// 创建空集合。
    pub const fn new(v: V) -> Self {
        Self {
            elements: v,
            _marker: PhantomData,
        }
    }

    /// 检查两个区间是否有交集
    #[inline]
    fn ranges_overlap<T1: Ord + Copy>(r1: &Range<T1>, r2: &Range<T1>) -> bool {
        !(r1.end <= r2.start || r1.start >= r2.end)
    }

    /// 分割区间：将原区间按分割范围分割成不重叠的部分
    ///
    /// 返回分割后的区间列表（最多2个区间）
    fn split_range(elem: &T, split_range: &Range<T::Type>) -> [Option<T>; 2] {
        let elem_range = elem.range();
        let has_left = elem_range.start < split_range.start;
        let has_right = elem_range.end > split_range.end;

        match (has_left, has_right) {
            (true, true) => {
                // 分裂成两段
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
                // 只保留左半段
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
                // 只保留右半段
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
            (false, false) => {
                // 完全被覆盖，不保留
                [None, None]
            }
        }
    }

    /// 返回内部区间的切片（已排序、已合并、互不重叠）。
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.elements
    }

    /// 返回区间迭代器（零拷贝）。
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.elements.iter()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn clear(&mut self) {
        self.elements.clear();
    }

    fn push_back<V2: VecOp<T>>(out: &mut V2, elem: T) -> Result<(), RangeError<T>> {
        out.push_back(elem).map_err(|_| RangeError::Capacity)
    }

    /// 将字节缓冲区转换为 T 类型的可变切片
    ///
    /// # Safety
    ///
    /// 调用者必须确保：
    /// - buffer 的对齐方式适合 T 类型
    /// - buffer 的大小至少为 size_of::<T>() * 所需元素数
    #[inline]
    fn bytes_to_slice_mut(buffer: &mut [u8]) -> &mut [T] {
        let len = buffer.len() / mem::size_of::<T>();
        let ptr = buffer.as_mut_ptr() as *mut T;
        // Safety: 调用者负责确保对齐和大小
        unsafe { slice::from_raw_parts_mut(ptr, len) }
    }

    /// 添加一个区间；会把与其重叠或相邻且 kind 相等的区间自动合并。
    /// 对于 kind 不同的重叠区间：
    /// - 如果旧区间可覆盖，则用新区间覆盖交集部分
    /// - 如果旧区间不可覆盖，则返回冲突错误
    ///
    /// # Arguments
    ///
    /// * `new_info` - 要添加的区间
    /// * `temp_buffer` - 临时内存缓冲区（字节数组），用于处理中间结果
    ///   建议大小为 `size_of::<T>() * capacity * (2-3)`，以应对区间分割情况
    ///
    /// # Errors
    ///
    /// - 如果容量不足，返回 `RangeSetError::Capacity`。
    /// - 如果与不可覆盖的区间冲突，返回 `RangeSetError::Conflict`。
    pub fn add(&mut self, new_info: T, temp_buffer: &mut [u8]) -> Result<(), RangeError<T>> {
        if new_info.range().start >= new_info.range().end {
            return Ok(());
        }

        for elem in self.iter() {
            // 如果没有交集，跳过
            if !Self::ranges_overlap(&elem.range(), &new_info.range()) {
                continue;
            }

            // 如果 kind 相同，跳过（稍后处理合并）
            if elem.kind() == new_info.kind() {
                continue;
            }

            // kind 不同且有交集：检查是否可覆盖
            if !elem.overwritable() {
                // 不可覆盖，返回冲突错误
                return Err(RangeError::Conflict {
                    new: new_info,
                    existing: elem.clone(),
                });
            }
        }

        // 所有冲突都可以覆盖，使用临时内存处理
        let temp_slice = Self::bytes_to_slice_mut(temp_buffer);
        let mut out = SliceVec::from_slice_len(temp_slice, 0);

        for elem in self.elements.drain(..) {
            // 如果没有交集，保留
            if !Self::ranges_overlap(&elem.range(), &new_info.range()) {
                out.push(elem);
                continue;
            }

            // 如果 kind 相同，稍后处理合并
            if elem.kind() == new_info.kind() {
                out.push(elem);
                continue;
            }

            // kind 不同且有交集：分割原区间（已经确认可覆盖）
            let split_parts = Self::split_range(&elem, &new_info.range());
            for part in split_parts.iter().flatten() {
                out.push(part.clone());
            }
        }

        // 将处理后的结果复制回原数组（正序）
        let out_len = out.len();
        for i in 0..out_len {
            Self::push_back(&mut self.elements, out[i].clone())?;
        }

        // 现在插入新区间，并与同 kind 的区间合并
        if self.elements.is_empty() {
            Self::push_back(&mut self.elements, new_info.clone())?;
            return Ok(());
        }

        // 二分查找插入位置
        let insert_at = self
            .elements
            .binary_search_by(|e| {
                if e.range().start < new_info.range().start {
                    core::cmp::Ordering::Less
                } else {
                    core::cmp::Ordering::Greater
                }
            })
            .unwrap_or_else(|pos| pos);

        let mut merged_range = new_info.range().clone();
        let mut insert_at = insert_at;

        // 向左合并：若左侧区间与 range 重叠/相邻且 kind 相等
        while insert_at > 0 {
            let left = &self.elements[insert_at - 1];
            if left.range().end < merged_range.start || left.kind() != new_info.kind() {
                break;
            }
            merged_range.start = min(merged_range.start, left.range().start);
            merged_range.end = max(merged_range.end, left.range().end);
            self.elements.remove(insert_at - 1);
            insert_at -= 1;
        }

        // 向右合并：若右侧区间与 range 重叠/相邻且 kind 相等
        while insert_at < self.elements.len() {
            let right = &self.elements[insert_at];
            if right.range().start > merged_range.end || right.kind() != new_info.kind() {
                break;
            }
            merged_range.start = min(merged_range.start, right.range().start);
            merged_range.end = max(merged_range.end, right.range().end);
            self.elements.remove(insert_at);
        }

        self.elements
            .insert(insert_at, new_info.clone_with_range(merged_range))
            .map_err(|_| RangeError::Capacity)?;
        Ok(())
    }

    /// 查询某个值是否落在任意一个区间中。
    #[inline]
    pub fn contains(&self, value: T::Type) -> bool {
        // 二分查找：找到可能包含 value 的区间
        self.elements
            .binary_search_by(|e| {
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

    /// 删除一个区间：从集合中移除与其相交的部分。
    ///
    /// 若被删除区间位于某个已有区间内部，会导致该已有区间被拆分为两段（保留原有 kind）。
    ///
    /// # Arguments
    ///
    /// * `range` - 要删除的区间
    /// * `temp_buffer` - 临时内存缓冲区（字节数组），用于处理中间结果
    ///
    /// # Errors
    ///
    /// 如果容量不足（删除操作导致区间分裂，新区间数量超过容量），返回 `RangeSetError::Capacity`。
    pub fn remove(
        &mut self,
        range: Range<T::Type>,
        temp_buffer: &mut [u8],
    ) -> Result<(), RangeError<T>> {
        if range.start >= range.end || self.elements.is_empty() {
            return Ok(());
        }

        let temp_slice = Self::bytes_to_slice_mut(temp_buffer);
        let mut out = SliceVec::from_slice_len(temp_slice, 0);
        for elem in self.elements.drain(..) {
            // 无交集
            if !Self::ranges_overlap(&elem.range(), &range) {
                out.push(elem);
                continue;
            }

            // 有交集，需要分割
            let split_parts = Self::split_range(&elem, &range);
            for part in split_parts.iter().flatten() {
                out.push(part.clone());
            }
        }

        // 将处理后的结果复制回原数组（正序）
        let out_len = out.len();
        for i in 0..out_len {
            Self::push_back(&mut self.elements, out[i].clone())?;
        }
        Ok(())
    }

    /// 批量添加多个区间。
    ///
    /// # Arguments
    ///
    /// * `ranges` - 要添加的区间集合
    /// * `temp_buffer` - 临时内存缓冲区（字节数组）
    ///
    /// # Errors
    ///
    /// 如果容量不足，返回 `RangeSetError::Capacity`。
    pub fn extend<I>(&mut self, ranges: I, temp_buffer: &mut [u8]) -> Result<(), RangeError<T>>
    where
        I: IntoIterator<Item = T>,
    {
        for v in ranges {
            self.add(v, temp_buffer)?;
        }
        Ok(())
    }
}

impl<T, const C: usize> VecOp<T> for heapless::Vec<T, C> {
    fn push_back(&mut self, item: T) -> Result<(), T> {
        self.push(item)
    }

    fn clear(&mut self) {
        self.clear();
    }

    fn insert(&mut self, index: usize, item: T) -> Result<(), T> {
        self.insert(index, item)
    }

    fn remove(&mut self, index: usize) -> T {
        self.remove(index)
    }

    fn drain<R>(&mut self, range: R) -> impl Iterator<Item = T>
    where
        R: RangeBounds<usize>,
    {
        self.drain(range)
    }
}

#[cfg(feature = "alloc")]
impl<T> VecOp<T> for alloc::vec::Vec<T> {
    fn push_back(&mut self, item: T) -> Result<(), T> {
        self.push(item);
        Ok(())
    }

    fn clear(&mut self) {
        self.clear();
    }

    fn insert(&mut self, index: usize, item: T) -> Result<(), T> {
        self.insert(index, item);
        Ok(())
    }

    fn remove(&mut self, index: usize) -> T {
        self.remove(index)
    }

    fn drain<R>(&mut self, range: R) -> impl Iterator<Item = T>
    where
        R: RangeBounds<usize>,
    {
        self.drain(range)
    }
}
