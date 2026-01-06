use core::cmp::{max, min};
use core::ops::Range;

use crate::{helpers, core_ops, RangeError, RangeInfo, RangeSetAllocOps};

/// 为 alloc::vec::Vec 实现 RangeSetAllocOps
#[cfg(feature = "alloc")]
impl<T: RangeInfo> RangeSetAllocOps<T> for alloc::vec::Vec<T> {
    fn merge_add(&mut self, new_info: T) -> Result<(), RangeError<T>> {
        if !core_ops::validate_range(&new_info) {
            return Ok(());
        }

        // 检查冲突
        core_ops::check_conflicts(self.iter(), &new_info)?;

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
        let insert_at = core_ops::find_insert_position(self, &new_info.range());

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
        core_ops::contains_point(self, value)
    }
}
