// 测试用的通用导入和辅助结构体
use core::fmt::Debug;
use core::ops::Range;
pub use ranges_ext::{RangeError, RangeInfo, RangeSet};

fn r(start: i32, end: i32) -> Range<i32> {
    start..end
}

// 简单的区间信息实现，用于测试
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestRange<T> {
    pub range: Range<T>,
    pub kind: (),
    pub overwritable: bool,
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

impl<T: Ord + Copy + Debug> RangeInfo for TestRange<T> {
    type Kind = ();
    type Type = T;

    fn range(&self) -> &Range<Self::Type> {
        &self.range
    }

    fn kind(&self) -> &Self::Kind {
        &self.kind
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

// 带有 kind 的区间信息实现
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestRangeWithKind<T, K> {
    pub range: Range<T>,
    pub kind: K,
    pub overwritable: bool,
}

impl<T, K> TestRangeWithKind<T, K> {
    pub fn new(range: Range<T>, kind: K, overwritable: bool) -> Self {
        Self {
            range,
            kind,
            overwritable,
        }
    }
}

impl<T: Ord + Copy + Debug, K: Debug + Eq + Clone> RangeInfo for TestRangeWithKind<T, K> {
    type Kind = K;
    type Type = T;

    fn range(&self) -> &Range<Self::Type> {
        &self.range
    }

    fn kind(&self) -> &Self::Kind {
        &self.kind
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