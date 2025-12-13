#![no_std]

use core::{
    cmp::{max, min},
    ops::Range,
};

use heapless::Vec;

/// 容量不足错误
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapacityError;

impl core::fmt::Display for CapacityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RangeSet capacity exceeded")
    }
}

impl core::error::Error for CapacityError {}

/// 一个「区间集合」数据结构：维护一组**有序、互不重叠**的半开区间 `[start, end)`。
///
/// - 插入时会把重叠/相邻的区间自动合并。
/// - 删除一个区间时，会从集合里移除交集；必要时把已有区间拆成左右两段。
///
/// 约定：空区间（`start >= end`）会被忽略。
#[derive(Clone, Debug)]
pub struct RangeSet<T, const C: usize = 128>
where
    T: Ord + Copy,
{
    elements: Vec<Range<T>, C>,
}

impl<T, const C: usize> Default for RangeSet<T, C>
where
    T: Ord + Copy,
{
    fn default() -> Self {
        Self {
            elements: Vec::new(),
        }
    }
}

impl<T, const C: usize> RangeSet<T, C>
where
    T: Ord + Copy,
{
    /// 创建空集合。
    pub fn new() -> Self {
        Self::default()
    }

    /// 返回内部区间的切片（已排序、已合并、互不重叠）。
    #[inline]
    pub fn as_slice(&self) -> &[Range<T>] {
        &self.elements
    }

    /// 返回区间迭代器（零拷贝）。
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Range<T>> {
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

    /// 添加一个区间；会把与其重叠或相邻的区间合并。
    ///
    /// # Errors
    ///
    /// 如果容量不足，返回 `CapacityError`。
    pub fn add(&mut self, range: Range<T>) -> Result<(), CapacityError> {
        if range.start >= range.end {
            return Ok(());
        }

        if self.elements.is_empty() {
            self.elements.push(range).map_err(|_| CapacityError)?;
            return Ok(());
        }

        // 二分查找插入位置：第一个 start >= range.start 的位置
        let insert_at = self
            .elements
            .binary_search_by(|e| {
                if e.start < range.start {
                    core::cmp::Ordering::Less
                } else {
                    core::cmp::Ordering::Greater
                }
            })
            .unwrap_or_else(|pos| pos);

        let mut merged_range = range;

        // 向左合并：若左侧区间与 range 重叠/相邻（end >= start）
        let mut insert_at = insert_at;
        while insert_at > 0 {
            let left = &self.elements[insert_at - 1];
            if left.end < merged_range.start {
                break;
            }
            merged_range.start = min(merged_range.start, left.start);
            merged_range.end = max(merged_range.end, left.end);
            self.elements.remove(insert_at - 1);
            insert_at -= 1;
        }

        // 向右合并：若右侧区间与 range 重叠/相邻（start <= end）
        while insert_at < self.elements.len() {
            let right = &self.elements[insert_at];
            if right.start > merged_range.end {
                break;
            }
            merged_range.start = min(merged_range.start, right.start);
            merged_range.end = max(merged_range.end, right.end);
            self.elements.remove(insert_at);
        }

        self.elements
            .insert(insert_at, merged_range)
            .map_err(|_| CapacityError)?;
        Ok(())
    }

    /// 查询某个值是否落在任意一个区间中。
    #[inline]
    pub fn contains(&self, value: T) -> bool {
        // 二分查找：找到可能包含 value 的区间
        self.elements
            .binary_search_by(|e| {
                if e.end <= value {
                    core::cmp::Ordering::Less
                } else if e.start > value {
                    core::cmp::Ordering::Greater
                } else {
                    core::cmp::Ordering::Equal
                }
            })
            .is_ok()
    }

    /// 删除一个区间：从集合中移除与其相交的部分。
    ///
    /// 若被删除区间位于某个已有区间内部，会导致该已有区间被拆分为两段。
    ///
    /// # Errors
    ///
    /// 如果容量不足（删除操作导致区间分裂，新区间数量超过容量），返回 `CapacityError`。
    pub fn remove(&mut self, range: Range<T>) -> Result<(), CapacityError> {
        if range.start >= range.end || self.elements.is_empty() {
            return Ok(());
        }

        let mut out: Vec<Range<T>, C> = Vec::new();
        for elem in self.elements.drain(..) {
            // 无交集
            if elem.end <= range.start || elem.start >= range.end {
                out.push(elem).map_err(|_| CapacityError)?;
                continue;
            }

            let has_left = elem.start < range.start;
            let has_right = elem.end > range.end;

            match (has_left, has_right) {
                (true, true) => {
                    // 需要分裂成两段
                    let left = elem.start..min(elem.end, range.start);
                    if left.start < left.end {
                        out.push(left).map_err(|_| CapacityError)?;
                    }
                    let right = max(elem.start, range.end)..elem.end;
                    if right.start < right.end {
                        out.push(right).map_err(|_| CapacityError)?;
                    }
                }
                (true, false) => {
                    // 只有左半段
                    let left = elem.start..min(elem.end, range.start);
                    if left.start < left.end {
                        out.push(left).map_err(|_| CapacityError)?;
                    }
                }
                (false, true) => {
                    // 只有右半段
                    let right = max(elem.start, range.end)..elem.end;
                    if right.start < right.end {
                        out.push(right).map_err(|_| CapacityError)?;
                    }
                }
                (false, false) => {
                    // 完全被删除，不添加任何内容
                }
            }
        }
        self.elements = out;
        Ok(())
    }

    /// 批量添加多个区间。
    ///
    /// # Errors
    ///
    /// 如果容量不足，返回 `CapacityError`。
    pub fn extend<I>(&mut self, ranges: I) -> Result<(), CapacityError>
    where
        I: IntoIterator<Item = Range<T>>,
    {
        for r in ranges {
            self.add(r)?;
        }
        Ok(())
    }
}
