#![allow(clippy::reversed_empty_ranges)]

mod common;
use common::*;

#[test]
fn test_empty_range_handling() {
    let mut set = RangeSet::<TestRange<i32>>::new();

    // 添加空区间（应该被忽略）
    let result = set.add(TestRange::new(10..10, true));
    assert!(result.is_ok());
    assert!(set.is_empty());

    // 添加反向区间（start > end）
    let result = set.add(TestRange::new(20..10, true));
    assert!(result.is_ok());
    assert!(set.is_empty());

    // 删除空区间（应该是 no-op）
    let result = set.remove(10..10);
    assert!(result.is_ok());

    // 删除反向区间（应该是 no-op）
    let result = set.remove(20..10);
    assert!(result.is_ok());
}

#[test]
fn test_boundary_overlap() {
    // 创建一个带有 kind 的测试结构体
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestRangeWithKind<T> {
        range: core::ops::Range<T>,
        kind: u32,
        overwritable: bool,
    }

    impl<T> TestRangeWithKind<T> {
        fn new(range: core::ops::Range<T>, kind: u32, overwritable: bool) -> Self {
            Self {
                range,
                kind,
                overwritable,
            }
        }
    }

    impl<T: core::fmt::Debug + Clone + Ord + Copy> RangeInfo for TestRangeWithKind<T> {
        type Kind = u32;
        type Type = T;

        fn range(&self) -> core::ops::Range<Self::Type> {
            self.range.clone()
        }

        fn kind(&self) -> &Self::Kind {
            &self.kind
        }

        fn overwritable(&self) -> bool {
            self.overwritable
        }

        fn clone_with_range(&self, range: core::ops::Range<Self::Type>) -> Self {
            Self {
                range,
                kind: self.kind,
                overwritable: self.overwritable,
            }
        }
    }

    let mut set = RangeSet::<TestRangeWithKind<i32>>::new();

    // 测试边界相接但不重叠的情况
    set.add(TestRangeWithKind::new(0..5, 1, true)).unwrap();
    set.add(TestRangeWithKind::new(5..10, 1, true)).unwrap(); // 应该合并（相邻且相同 kind）
    set.add(TestRangeWithKind::new(10..15, 2, true)).unwrap(); // kind 不同，不应合并

    assert_eq!(set.len(), 2);
    assert_eq!(set.as_slice()[0].range(), (0..10));
    assert_eq!(set.as_slice()[1].range(), (10..15));

    // 测试有间隙的情况
    set.clear();
    set.add(TestRangeWithKind::new(0..5, 1, true)).unwrap();
    set.add(TestRangeWithKind::new(7..10, 1, true)).unwrap(); // 有间隙，不应合并

    assert_eq!(set.len(), 2);
    assert_eq!(set.as_slice()[0].range(), (0..5));
    assert_eq!(set.as_slice()[1].range(), (7..10));
}

#[test]
fn test_single_point_ranges() {
    let mut set = RangeSet::<TestRange<i32>>::new();

    // 添加单点区间
    set.add(TestRange::new(5..6, true)).unwrap();
    set.add(TestRange::new(6..7, true)).unwrap(); // 应该合并

    assert_eq!(set.len(), 1);
    assert_eq!(set.as_slice()[0].range(), (5..7));

    // 删除单点
    set.remove(6..6).unwrap(); // 删除空区间，应该 no-op
    assert_eq!(set.len(), 1);

    set.remove(5..6).unwrap(); // 删除第一个点
    assert_eq!(set.len(), 1);
    assert_eq!(set.as_slice()[0].range(), (6..7));
}

#[test]
fn test_extreme_values() {
    let mut set = RangeSet::<TestRange<i32>>::new();

    // 测试极值
    set.add(TestRange::new(i32::MIN..i32::MAX, true)).unwrap();
    assert_eq!(set.len(), 1);

    // 尝试覆盖整个范围
    set.add(TestRange::new(i32::MIN..i32::MAX, true)).unwrap();
    assert_eq!(set.len(), 1);

    // 测试 contains 极值
    assert!(set.contains(i32::MIN));
    assert!(set.contains(i32::MAX - 1));
    assert!(!set.contains(i32::MAX));
}
