// 测试用的通用导入和辅助结构体
use core::{fmt::Debug, ops::Range};
pub use ranges_ext::*;

// 简单的区间信息实现，用于测试
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestRange<T> {
    pub range: Range<T>,
    pub kind: (),
    pub overwritable: bool,
}

impl<T: Default> Default for TestRange<T> {
    fn default() -> Self {
        Self {
            range: T::default()..T::default(),
            kind: (),
            overwritable: false,
        }
    }
}

impl<T> TestRange<T> {
    pub fn new(range: Range<T>, overwritable: bool) -> Self {
        Self {
            range,
            kind: (),
            overwritable,
        }
    }
}

impl<T: Ord + Copy + Debug + Default> RangeInfo for TestRange<T> {
    type Kind = ();
    type Type = T;

    fn range(&self) -> Range<Self::Type> {
        self.range.clone()
    }

    fn kind(&self) -> Self::Kind {
        self.kind
    }

    fn overwritable(&self) -> bool {
        self.overwritable
    }

    fn clone_with_range(&self, range: Range<Self::Type>) -> Self {
        Self {
            range,
            kind: self.kind,
            overwritable: self.overwritable,
        }
    }
}

#[allow(unused)]
// 带有 kind 的区间信息实现
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestRangeWithKind<T, K> {
    pub range: Range<T>,
    pub kind: K,
    pub overwritable: bool,
}

impl<T: Default, K: Default> Default for TestRangeWithKind<T, K> {
    fn default() -> Self {
        Self {
            range: T::default()..T::default(),
            kind: K::default(),
            overwritable: false,
        }
    }
}

#[allow(unused)]
impl<T, K> TestRangeWithKind<T, K> {
    pub fn new(range: Range<T>, kind: K, overwritable: bool) -> Self {
        Self {
            range,
            kind,
            overwritable,
        }
    }
}

impl<T: Ord + Copy + Debug + Default, K: Debug + Eq + Clone + Default> RangeInfo
    for TestRangeWithKind<T, K>
{
    type Kind = K;
    type Type = T;

    fn range(&self) -> Range<Self::Type> {
        self.range.clone()
    }

    fn kind(&self) -> Self::Kind {
        self.kind.clone()
    }

    fn overwritable(&self) -> bool {
        self.overwritable
    }

    fn clone_with_range(&self, range: Range<Self::Type>) -> Self {
        Self {
            range,
            kind: self.kind.clone(),
            overwritable: self.overwritable,
        }
    }
}

// 辅助函数，提供固定大小的临时缓冲区
pub fn temp_buffer() -> [u8; 1024] {
    [0u8; 1024]
}

// 扩展 trait，为 heapless::Vec 提供方便的测试方法
pub trait TestRangeSetOps<T: RangeInfo> {
    fn test_add(&mut self, info: T) -> Result<(), RangeError<T>>;
    fn test_remove(&mut self, range: Range<T::Type>) -> Result<(), RangeError<T>>;
    fn test_extend<I>(&mut self, ranges: I) -> Result<(), RangeError<T>>
    where
        I: IntoIterator<Item = T>;
    fn test_contains_point(&self, value: T::Type) -> bool;
}

impl<T: RangeInfo, const N: usize> TestRangeSetOps<T> for heapless::Vec<T, N> {
    fn test_add(&mut self, info: T) -> Result<(), RangeError<T>> {
        let mut buffer = temp_buffer();
        self.merge_add(info, &mut buffer)
    }

    fn test_remove(&mut self, range: Range<T::Type>) -> Result<(), RangeError<T>> {
        let mut buffer = temp_buffer();
        self.merge_remove(range, &mut buffer)
    }

    fn test_extend<I>(&mut self, ranges: I) -> Result<(), RangeError<T>>
    where
        I: IntoIterator<Item = T>,
    {
        let mut buffer = temp_buffer();
        self.merge_extend(ranges, &mut buffer)
    }

    fn test_contains_point(&self, value: T::Type) -> bool {
        self.merge_contains_point(value)
    }
}
