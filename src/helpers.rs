use core::{mem, ops::Range, slice};

use crate::{RangeInfo, SliceVec};

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
            let left = elem_range.start..core::cmp::min(elem_range.end, split_range.start);
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
            let right = core::cmp::max(elem_range.start, split_range.end)..elem_range.end;
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
