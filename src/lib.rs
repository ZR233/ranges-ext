#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::cmp::{max, min};
use core::ops::Range;

#[cfg(test)]
extern crate std;

/// 原始区间与 metadata 的配对。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OriginalRange<T, M> {
    pub range: Range<T>,
    pub meta: M,
}

/// 合并后的区间元素：包含合并后的 range 和所有原始区间列表。
#[derive(Clone)]
pub struct MergedRange<T, M> {
    /// 合并后的区间范围
    pub merged: Range<T>,
    /// 该合并区间包含的所有原始区间及其 metadata
    pub originals: Vec<OriginalRange<T, M>>,
}

impl<T, M> core::fmt::Debug for MergedRange<T, M>
where
    T: core::fmt::Debug,
    M: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MergedRange")
            .field(
                "merged",
                &format_args!("[{:?}..{:?})", self.merged.start, self.merged.end),
            )
            .field("originals", &OriginalsList(&self.originals))
            .finish()
    }
}

/// 辅助结构，用于格式化原始区间列表
struct OriginalsList<'a, T, M>(&'a [OriginalRange<T, M>]);

impl<T, M> core::fmt::Debug for OriginalsList<'_, T, M>
where
    T: core::fmt::Debug,
    M: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        for orig in self.0 {
            list.entry(&format_args!(
                "[{:?}..{:?}) → {:?}",
                orig.range.start, orig.range.end, orig.meta
            ));
        }
        list.finish()
    }
}

impl<T, M> core::fmt::Display for MergedRange<T, M>
where
    T: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[{}..{})", self.merged.start, self.merged.end)
    }
}

impl<T, M> core::fmt::LowerHex for MergedRange<T, M>
where
    T: core::fmt::LowerHex,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "[{:#x}..{:#x})", self.merged.start, self.merged.end)
        } else {
            write!(f, "[{:x}..{:x})", self.merged.start, self.merged.end)
        }
    }
}

impl<T, M> core::fmt::UpperHex for MergedRange<T, M>
where
    T: core::fmt::UpperHex,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "[{:#X}..{:#X})", self.merged.start, self.merged.end)
        } else {
            write!(f, "[{:X}..{:X})", self.merged.start, self.merged.end)
        }
    }
}

impl<T, M> core::fmt::Binary for MergedRange<T, M>
where
    T: core::fmt::Binary,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "[{:#b}..{:#b})", self.merged.start, self.merged.end)
        } else {
            write!(f, "[{:b}..{:b})", self.merged.start, self.merged.end)
        }
    }
}

impl<T, M> core::fmt::Octal for MergedRange<T, M>
where
    T: core::fmt::Octal,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "[{:#o}..{:#o})", self.merged.start, self.merged.end)
        } else {
            write!(f, "[{:o}..{:o})", self.merged.start, self.merged.end)
        }
    }
}

/// 一个「区间集合」数据结构：维护一组**有序、互不重叠**的半开区间 `[start, end)`。
///
/// - 插入时会把重叠/相邻的区间自动合并。
/// - 每个合并后的区间会保留其包含的所有原始区间及 metadata。
/// - 删除一个区间时，会从集合里移除交集；必要时把已有区间拆成左右两段。
/// - 删除时会同步移除完全被删除的原始区间。
///
/// 约定：空区间（`start >= end`）会被忽略。
#[derive(Clone, Debug)]
pub struct RangeSet<T, M = ()>
where
    T: Ord + Copy,
{
    elements: Vec<MergedRange<T, M>>,
}

impl<T, M> Default for RangeSet<T, M>
where
    T: Ord + Copy,
{
    fn default() -> Self {
        Self {
            elements: Vec::new(),
        }
    }
}

impl<T, M> RangeSet<T, M>
where
    T: Ord + Copy,
{
    /// 创建空集合。
    pub fn new() -> Self {
        Self::default()
    }

    /// 返回内部元素的切片（每个元素包含合并后的 range 和原始列表）。
    #[inline]
    pub fn elements(&self) -> &[MergedRange<T, M>] {
        &self.elements
    }

    /// 返回归一化后的区间切片（仅 range，已排序、已合并、互不重叠）。
    pub fn as_slice(&self) -> Vec<Range<T>> {
        self.elements.iter().map(|e| e.merged.clone()).collect()
    }

    /// 返回归一化后的区间迭代器（零拷贝）。
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Range<T>> {
        self.elements.iter().map(|e| &e.merged)
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

    /// 添加一个区间及其 metadata；会把与其重叠或相邻的区间合并。
    pub fn add(&mut self, range: Range<T>, meta: M)
    where
        M: PartialEq,
    {
        if range.start >= range.end {
            return;
        }

        let original = OriginalRange {
            range: range.clone(),
            meta,
        };

        if self.elements.is_empty() {
            self.elements.push(MergedRange {
                merged: range,
                originals: alloc::vec![original],
            });
            return;
        }

        // 二分查找插入位置：第一个 start >= range.start 的位置
        let insert_at = self
            .elements
            .binary_search_by(|e| {
                if e.merged.start < range.start {
                    core::cmp::Ordering::Less
                } else {
                    core::cmp::Ordering::Greater
                }
            })
            .unwrap_or_else(|pos| pos);

        let mut merged_range = range;
        let mut merged_originals = alloc::vec![original];

        // 向左合并：若左侧区间与 range 重叠/相邻（end >= start）
        let mut insert_at = insert_at;
        while insert_at > 0 {
            let left = &self.elements[insert_at - 1];
            if left.merged.end < merged_range.start {
                break;
            }
            merged_range.start = min(merged_range.start, left.merged.start);
            merged_range.end = max(merged_range.end, left.merged.end);
            let left_elem = self.elements.remove(insert_at - 1);
            merged_originals.reserve(left_elem.originals.len());
            merged_originals.extend(left_elem.originals);
            insert_at -= 1;
        }

        // 向右合并：若右侧区间与 range 重叠/相邻（start <= end）
        while insert_at < self.elements.len() {
            let right = &self.elements[insert_at];
            if right.merged.start > merged_range.end {
                break;
            }
            merged_range.start = min(merged_range.start, right.merged.start);
            merged_range.end = max(merged_range.end, right.merged.end);
            let right_elem = self.elements.remove(insert_at);
            merged_originals.reserve(right_elem.originals.len());
            merged_originals.extend(right_elem.originals);
        }

        // 合并原始区间：对于 metadata 相等且相邻/重叠的原始区间进行合并
        merged_originals = Self::merge_originals(merged_originals);

        self.elements.insert(
            insert_at,
            MergedRange {
                merged: merged_range,
                originals: merged_originals,
            },
        );
    }

    /// 合并原始区间列表：对于 metadata 相等且相邻/重叠的原始区间进行合并
    fn merge_originals(mut originals: Vec<OriginalRange<T, M>>) -> Vec<OriginalRange<T, M>>
    where
        M: PartialEq,
    {
        if originals.len() <= 1 {
            return originals;
        }

        // 按区间起点排序（使用不稳定排序提升性能）
        originals.sort_unstable_by(|a, b| a.range.start.cmp(&b.range.start));

        let mut result = Vec::with_capacity(originals.len());
        let mut iter = originals.into_iter();
        let mut current = iter.next().unwrap();

        for next in iter {
            // 如果 metadata 相等且区间相邻或重叠，则合并
            if current.meta == next.meta && current.range.end >= next.range.start {
                current.range.end = max(current.range.end, next.range.end);
            } else {
                result.push(current);
                current = next;
            }
        }
        result.push(current);

        result
    }

    /// 查询某个值是否落在任意一个区间中。
    #[inline]
    pub fn contains(&self, value: T) -> bool {
        // 二分查找：找到可能包含 value 的区间
        self.elements
            .binary_search_by(|e| {
                if e.merged.end <= value {
                    core::cmp::Ordering::Less
                } else if e.merged.start > value {
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
    /// 同时会移除完全被删除的原始区间。
    pub fn remove_range(&mut self, range: Range<T>)
    where
        M: Clone,
    {
        if range.start >= range.end || self.elements.is_empty() {
            return;
        }

        let mut out: Vec<MergedRange<T, M>> = Vec::with_capacity(self.elements.len() + 1);
        for elem in self.elements.drain(..) {
            // 无交集
            if elem.merged.end <= range.start || elem.merged.start >= range.end {
                out.push(elem);
                continue;
            }

            // 过滤原始区间：移除完全被删除范围包含的原始区间
            let filtered_originals: Vec<_> = elem
                .originals
                .into_iter()
                .filter(|orig| {
                    // 如果原始区间完全在删除范围内，则丢弃
                    !(range.start <= orig.range.start && orig.range.end <= range.end)
                })
                .collect();

            // 如果所有原始区间都被删除，跳过该元素
            if filtered_originals.is_empty() {
                continue;
            }

            let has_left = elem.merged.start < range.start;
            let has_right = elem.merged.end > range.end;

            match (has_left, has_right) {
                (true, true) => {
                    // 需要分裂成两段，左右两边都需要克隆
                    let left = elem.merged.start..min(elem.merged.end, range.start);
                    if left.start < left.end {
                        out.push(MergedRange {
                            merged: left,
                            originals: filtered_originals.clone(),
                        });
                    }
                    let right = max(elem.merged.start, range.end)..elem.merged.end;
                    if right.start < right.end {
                        out.push(MergedRange {
                            merged: right,
                            originals: filtered_originals,
                        });
                    }
                }
                (true, false) => {
                    // 只有左半段
                    let left = elem.merged.start..min(elem.merged.end, range.start);
                    if left.start < left.end {
                        out.push(MergedRange {
                            merged: left,
                            originals: filtered_originals,
                        });
                    }
                }
                (false, true) => {
                    // 只有右半段
                    let right = max(elem.merged.start, range.end)..elem.merged.end;
                    if right.start < right.end {
                        out.push(MergedRange {
                            merged: right,
                            originals: filtered_originals,
                        });
                    }
                }
                (false, false) => {
                    // 不应该到达这里，因为上面已经检查了无交集的情况
                }
            }
        }
        self.elements = out;
    }
}

impl<T> RangeSet<T, ()>
where
    T: Ord + Copy,
{
    /// 添加一个区间（不带 metadata）。
    pub fn add_range(&mut self, range: Range<T>) {
        self.add(range, ());
    }

    /// 批量添加多个区间（不带 metadata）。
    pub fn extend<I>(&mut self, ranges: I)
    where
        I: IntoIterator<Item = Range<T>>,
    {
        for r in ranges {
            self.add_range(r);
        }
    }
}
