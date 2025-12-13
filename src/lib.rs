#![no_std]

use core::{
    cmp::{max, min},
    ops::Range,
};

use heapless::Vec;

/// RangeSet 错误类型
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RangeError<T, K>
where
    T: Ord + Copy,
    K: core::fmt::Debug + Eq + Clone,
{
    /// 容量不足错误
    #[error("RangeSet capacity exceeded")]
    Capacity,
    /// 区间冲突错误：尝试覆盖不可覆盖的区间
    #[error("Range conflict: new {new:?} conflicts with existing non-overwritable {existing:?}")]
    Conflict {
        /// 新添加的区间
        new: OneRange<T, K>,
        /// 已存在的冲突区间
        existing: OneRange<T, K>,
    },
}

/// 区间信息：包含范围和自定义元数据
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OneRange<T, K = ()>
where
    T: Ord + Copy,
    K: core::fmt::Debug + Eq + Clone,
{
    pub range: Range<T>,
    pub kind: K,
    /// 是否可被其他 kind 的区间覆盖
    pub overwritable: bool,
}

impl<T, K> OneRange<T, K>
where
    T: Ord + Copy,
    K: core::fmt::Debug + Eq + Clone,
{
    pub fn new(range: Range<T>, kind: K, overwritable: bool) -> Self {
        Self {
            range,
            kind,
            overwritable,
        }
    }
}

/// 一个「区间集合」数据结构：维护一组**有序、互不重叠**的半开区间 `[start, end)`。
///
/// - 插入时会把重叠/相邻且 kind 相等的区间自动合并。
/// - 删除一个区间时，会从集合里移除交集；必要时把已有区间拆成左右两段。
///
/// 约定：空区间（`start >= end`）会被忽略。
#[derive(Clone, Debug)]
pub struct RangeSet<T, K = (), const C: usize = 128>
where
    T: Ord + Copy,
    K: core::fmt::Debug + Eq + Clone,
{
    elements: Vec<OneRange<T, K>, C>,
}

impl<T, K, const C: usize> Default for RangeSet<T, K, C>
where
    T: Ord + Copy,
    K: core::fmt::Debug + Eq + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, K, const C: usize> RangeSet<T, K, C>
where
    T: Ord + Copy,
    K: core::fmt::Debug + Eq + Clone,
{
    /// 创建空集合。
    pub const fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// 返回内部区间的切片（已排序、已合并、互不重叠）。
    #[inline]
    pub fn as_slice(&self) -> &[OneRange<T, K>] {
        &self.elements
    }

    /// 返回区间迭代器（零拷贝）。
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &OneRange<T, K>> {
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

    /// 添加一个区间；会把与其重叠或相邻且 kind 相等的区间自动合并。
    /// 对于 kind 不同的重叠区间：
    /// - 如果旧区间可覆盖，则用新区间覆盖交集部分
    /// - 如果旧区间不可覆盖，则返回冲突错误
    ///
    /// # Errors
    ///
    /// - 如果容量不足，返回 `RangeSetError::Capacity`。
    /// - 如果与不可覆盖的区间冲突，返回 `RangeSetError::Conflict`。
    pub fn add(
        &mut self,
        range: Range<T>,
        kind: K,
        overwritable: bool,
    ) -> Result<(), RangeError<T, K>> {
        if range.start >= range.end {
            return Ok(());
        }

        // 先检查是否有不可覆盖的冲突
        let new_info = OneRange::new(range.clone(), kind.clone(), overwritable);

        for elem in &self.elements {
            // 如果没有交集，跳过
            if elem.range.end <= range.start || elem.range.start >= range.end {
                continue;
            }

            // 如果 kind 相同，跳过（稍后处理合并）
            if elem.kind == kind {
                continue;
            }

            // kind 不同且有交集：检查是否可覆盖
            if !elem.overwritable {
                // 不可覆盖，返回冲突错误
                return Err(RangeError::Conflict {
                    new: new_info,
                    existing: elem.clone(),
                });
            }
        }

        // 所有冲突都可以覆盖，现在开始处理
        let mut out: Vec<OneRange<T, K>, C> = Vec::new();

        for elem in self.elements.drain(..) {
            // 如果没有交集，保留
            if elem.range.end <= range.start || elem.range.start >= range.end {
                out.push(elem).map_err(|_| RangeError::Capacity)?;
                continue;
            }

            // 如果 kind 相同，稍后处理合并
            if elem.kind == kind {
                out.push(elem).map_err(|_| RangeError::Capacity)?;
                continue;
            }

            // kind 不同且有交集：分割原区间（已经确认可覆盖）

            // 可覆盖：需要分割原区间
            let has_left = elem.range.start < range.start;
            let has_right = elem.range.end > range.end;

            match (has_left, has_right) {
                (true, true) => {
                    // 分裂成两段
                    let left = elem.range.start..range.start;
                    if left.start < left.end {
                        out.push(OneRange::new(left, elem.kind.clone(), elem.overwritable))
                            .map_err(|_| RangeError::Capacity)?;
                    }
                    let right = range.end..elem.range.end;
                    if right.start < right.end {
                        out.push(OneRange::new(right, elem.kind, elem.overwritable))
                            .map_err(|_| RangeError::Capacity)?;
                    }
                }
                (true, false) => {
                    // 只保留左半段
                    let left = elem.range.start..range.start;
                    if left.start < left.end {
                        out.push(OneRange::new(left, elem.kind, elem.overwritable))
                            .map_err(|_| RangeError::Capacity)?;
                    }
                }
                (false, true) => {
                    // 只保留右半段
                    let right = range.end..elem.range.end;
                    if right.start < right.end {
                        out.push(OneRange::new(right, elem.kind, elem.overwritable))
                            .map_err(|_| RangeError::Capacity)?;
                    }
                }
                (false, false) => {
                    // 完全被覆盖，不保留
                }
            }
        }

        self.elements = out;

        // 现在插入新区间，并与同 kind 的区间合并
        if self.elements.is_empty() {
            self.elements
                .push(OneRange::new(range, kind, overwritable))
                .map_err(|_| RangeError::Capacity)?;
            return Ok(());
        }

        // 二分查找插入位置
        let insert_at = self
            .elements
            .binary_search_by(|e| {
                if e.range.start < range.start {
                    core::cmp::Ordering::Less
                } else {
                    core::cmp::Ordering::Greater
                }
            })
            .unwrap_or_else(|pos| pos);

        let mut merged_range = range;
        let mut insert_at = insert_at;

        // 向左合并：若左侧区间与 range 重叠/相邻且 kind 相等
        while insert_at > 0 {
            let left = &self.elements[insert_at - 1];
            if left.range.end < merged_range.start || left.kind != kind {
                break;
            }
            merged_range.start = min(merged_range.start, left.range.start);
            merged_range.end = max(merged_range.end, left.range.end);
            self.elements.remove(insert_at - 1);
            insert_at -= 1;
        }

        // 向右合并：若右侧区间与 range 重叠/相邻且 kind 相等
        while insert_at < self.elements.len() {
            let right = &self.elements[insert_at];
            if right.range.start > merged_range.end || right.kind != kind {
                break;
            }
            merged_range.start = min(merged_range.start, right.range.start);
            merged_range.end = max(merged_range.end, right.range.end);
            self.elements.remove(insert_at);
        }

        self.elements
            .insert(insert_at, OneRange::new(merged_range, kind, overwritable))
            .map_err(|_| RangeError::Capacity)?;
        Ok(())
    }

    /// 查询某个值是否落在任意一个区间中。
    #[inline]
    pub fn contains(&self, value: T) -> bool {
        // 二分查找：找到可能包含 value 的区间
        self.elements
            .binary_search_by(|e| {
                if e.range.end <= value {
                    core::cmp::Ordering::Less
                } else if e.range.start > value {
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
    /// # Errors
    ///
    /// 如果容量不足（删除操作导致区间分裂，新区间数量超过容量），返回 `RangeSetError::Capacity`。
    pub fn remove(&mut self, range: Range<T>) -> Result<(), RangeError<T, K>> {
        if range.start >= range.end || self.elements.is_empty() {
            return Ok(());
        }

        let mut out: Vec<OneRange<T, K>, C> = Vec::new();
        for elem in self.elements.drain(..) {
            // 无交集
            if elem.range.end <= range.start || elem.range.start >= range.end {
                out.push(elem).map_err(|_| RangeError::Capacity)?;
                continue;
            }

            let has_left = elem.range.start < range.start;
            let has_right = elem.range.end > range.end;

            match (has_left, has_right) {
                (true, true) => {
                    // 需要分裂成两段
                    let left = elem.range.start..min(elem.range.end, range.start);
                    if left.start < left.end {
                        out.push(OneRange::new(left, elem.kind.clone(), elem.overwritable))
                            .map_err(|_| RangeError::Capacity)?;
                    }
                    let right = max(elem.range.start, range.end)..elem.range.end;
                    if right.start < right.end {
                        out.push(OneRange::new(right, elem.kind, elem.overwritable))
                            .map_err(|_| RangeError::Capacity)?;
                    }
                }
                (true, false) => {
                    // 只有左半段
                    let left = elem.range.start..min(elem.range.end, range.start);
                    if left.start < left.end {
                        out.push(OneRange::new(left, elem.kind, elem.overwritable))
                            .map_err(|_| RangeError::Capacity)?;
                    }
                }
                (false, true) => {
                    // 只有右半段
                    let right = max(elem.range.start, range.end)..elem.range.end;
                    if right.start < right.end {
                        out.push(OneRange::new(right, elem.kind, elem.overwritable))
                            .map_err(|_| RangeError::Capacity)?;
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
    /// 如果容量不足，返回 `RangeSetError::Capacity`。
    pub fn extend<I>(&mut self, ranges: I) -> Result<(), RangeError<T, K>>
    where
        I: IntoIterator<Item = (Range<T>, K, bool)>,
    {
        for (r, kind, overwritable) in ranges {
            self.add(r, kind, overwritable)?;
        }
        Ok(())
    }
}
