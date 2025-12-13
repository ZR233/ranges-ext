#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::cmp::{max, min};
use core::ops::Range;

/// 一个「区间集合」数据结构：维护一组**有序、互不重叠**的半开区间 `[start, end)`。
///
/// - 插入时会把重叠/相邻的区间自动合并。
/// - 删除一个区间时，会从集合里移除交集；必要时把已有区间拆成左右两段。
///
/// 约定：空区间（`start >= end`）会被忽略。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RangeSet<T>
where
    T: Ord + Copy,
{
    ranges: Vec<Range<T>>,
}

impl<T> RangeSet<T>
where
    T: Ord + Copy,
{
    /// 创建空集合。
    pub fn new() -> Self {
        Self { ranges: Vec::new() }
    }

    /// 返回内部归一化后的区间切片（已排序、已合并、互不重叠）。
    pub fn as_slice(&self) -> &[Range<T>] {
        &self.ranges
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    pub fn clear(&mut self) {
        self.ranges.clear();
    }

    /// 添加一个区间；会把与其重叠或相邻的区间合并。
    pub fn add_range(&mut self, mut range: Range<T>) {
        if range.start >= range.end {
            return;
        }

        if self.ranges.is_empty() {
            self.ranges.push(range);
            return;
        }

        // 找到第一个 start 大于等于 range.start 的位置
        let mut insert_at = 0usize;
        while insert_at < self.ranges.len() && self.ranges[insert_at].start < range.start {
            insert_at += 1;
        }

        // 向左合并：若左侧区间与 range 重叠/相邻（end >= start）
        while insert_at > 0 {
            let left = &self.ranges[insert_at - 1];
            if left.end < range.start {
                break;
            }
            range.start = min(range.start, left.start);
            range.end = max(range.end, left.end);
            self.ranges.remove(insert_at - 1);
            insert_at -= 1;
        }

        // 向右合并：若右侧区间与 range 重叠/相邻（start <= end）
        while insert_at < self.ranges.len() {
            let right = &self.ranges[insert_at];
            if right.start > range.end {
                break;
            }
            range.start = min(range.start, right.start);
            range.end = max(range.end, right.end);
            self.ranges.remove(insert_at);
        }

        self.ranges.insert(insert_at, range);
    }

    /// 批量添加多个区间。
    pub fn extend<I>(&mut self, ranges: I)
    where
        I: IntoIterator<Item = Range<T>>,
    {
        for r in ranges {
            self.add_range(r);
        }
    }

    /// 查询某个值是否落在任意一个区间中。
    pub fn contains(&self, value: T) -> bool {
        // 二分查找最后一个 start <= value 的区间
        let mut lo = 0usize;
        let mut hi = self.ranges.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.ranges[mid].start <= value {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        if lo == 0 {
            return false;
        }
        let r = &self.ranges[lo - 1];
        r.start <= value && value < r.end
    }

    /// 删除一个区间：从集合中移除与其相交的部分。
    ///
    /// 若被删除区间位于某个已有区间内部，会导致该已有区间被拆分为两段。
    pub fn remove_range(&mut self, range: Range<T>) {
        if range.start >= range.end || self.ranges.is_empty() {
            return;
        }

        let mut out: Vec<Range<T>> = Vec::with_capacity(self.ranges.len() + 1);
        for r in self.ranges.drain(..) {
            // 无交集
            if r.end <= range.start || r.start >= range.end {
                out.push(r);
                continue;
            }

            // 左半段保留: [r.start, range.start)
            if r.start < range.start {
                let left = r.start..min(r.end, range.start);
                if left.start < left.end {
                    out.push(left);
                }
            }

            // 右半段保留: [range.end, r.end)
            if r.end > range.end {
                let right = max(r.start, range.end)..r.end;
                if right.start < right.end {
                    out.push(right);
                }
            }
        }
        self.ranges = out;
    }

    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&Range<T>, &Range<T>) -> core::cmp::Ordering,
    {
        self.ranges.sort_by(|a, b| compare(a, b));
    }
}
