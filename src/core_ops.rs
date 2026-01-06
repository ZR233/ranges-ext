use core::ops::Range;

use crate::{RangeError, RangeInfo};

/// 验证区间有效性
#[inline]
pub fn validate_range<T: RangeInfo>(info: &T) -> bool {
    info.range().start < info.range().end
}

/// 检查区间冲突
pub fn check_conflicts<'a, T: RangeInfo + 'a, I: IntoIterator<Item = &'a T>>(
    elements: I,
    new_info: &T,
) -> Result<(), RangeError<T>> {
    let new_range = new_info.range();
    let new_kind = new_info.kind();

    for elem in elements {
        let elem_range = elem.range();

        if !crate::helpers::ranges_overlap(&elem_range, &new_range) {
            continue;
        }

        if elem.kind() == new_kind {
            continue;
        }

        if !elem.overwritable() {
            return Err(RangeError::Conflict {
                new: new_info.clone(),
                existing: elem.clone(),
            });
        }
    }

    Ok(())
}

/// 查找插入位置（二分查找）
pub fn find_insert_position<T: RangeInfo>(
    elements: &[T],
    new_range: &Range<T::Type>,
) -> usize {
    elements
        .binary_search_by(|e| {
            if e.range().start < new_range.start {
                core::cmp::Ordering::Less
            } else {
                core::cmp::Ordering::Greater
            }
        })
        .unwrap_or_else(|pos| pos)
}

/// 检查点是否包含在任意区间中
pub fn contains_point<T: RangeInfo>(elements: &[T], value: T::Type) -> bool {
    elements
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
