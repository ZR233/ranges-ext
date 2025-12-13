mod common;
use common::*;

#[test]
fn test_merge_complex_scenarios() {
    // 测试复杂的合并场景：新区间与多个相同kind的区间重叠
    let mut set = RangeSet::<TestRangeWithKind<i32, i32>>::new();

    set.add(TestRangeWithKind::new(0..5, 1, true)).unwrap();
    set.add(TestRangeWithKind::new(10..15, 1, true)).unwrap();
    set.add(TestRangeWithKind::new(20..25, 2, true)).unwrap(); // 不同 kind

    // 添加一个能桥接前两个区间的区间
    set.add(TestRangeWithKind::new(3..22, 1, true)).unwrap();

    // 期望：前两个区间和新区间合并成一个，第三个区间被分割
    assert_eq!(set.len(), 2);
    assert_eq!(set.as_slice()[0].range(), &(0..22));
    assert_eq!(set.as_slice()[0].kind(), &1);
    assert_eq!(set.as_slice()[1].range(), &(22..25));
    assert_eq!(set.as_slice()[1].kind(), &2);
}

#[test]
fn test_remove_split_edge_case() {
    // 测试删除操作导致的分裂边界情况
    let mut set = RangeSet::<TestRange<i32>>::new();

    set.add(TestRange::new(0..10, true)).unwrap();

    // 删除边界上的单点
    set.remove(5..6).unwrap();

    assert_eq!(set.len(), 2);
    assert_eq!(set.as_slice()[0].range(), &(0..5));
    assert_eq!(set.as_slice()[1].range(), &(6..10));

    // 删除整个左半部分
    set.remove(0..5).unwrap();

    assert_eq!(set.len(), 1);
    assert_eq!(set.as_slice()[0].range(), &(6..10));
}

#[test]
fn test_capacity_overflow_simple() {
    // 测试简单的容量溢出
    let mut set: RangeSet<TestRange<i32>, 2> = RangeSet::new();

    set.add(TestRange::new(0..5, true)).unwrap();
    set.add(TestRange::new(10..15, true)).unwrap();
    assert_eq!(set.len(), 2);

    // 添加第三个不重叠的区间应该失败
    let result = set.add(TestRange::new(20..25, true));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), RangeError::Capacity);
}

#[test]
fn test_contains_binary_search_edge() {
    // 测试 contains 方法在边界情况下的行为
    let mut set = RangeSet::<TestRange<i32>>::new();

    set.add(TestRange::new(10..20, true)).unwrap();
    set.add(TestRange::new(30..40, true)).unwrap();

    // 测试边界值
    assert!(set.contains(10));
    assert!(set.contains(19));
    assert!(!set.contains(20));
    assert!(!set.contains(29));
    assert!(set.contains(30));
    assert!(!set.contains(40));

    // 测试极值
    assert!(!set.contains(i32::MIN));
    assert!(!set.contains(i32::MAX));
}

#[test]
fn test_adjacent_but_different_kind() {
    // 测试相邻但不同 kind 的区间
    let mut set = RangeSet::<TestRangeWithKind<i32, i32>>::new();

    set.add(TestRangeWithKind::new(0..5, 1, true)).unwrap();
    set.add(TestRangeWithKind::new(5..10, 2, true)).unwrap(); // 相邻但不同 kind
    set.add(TestRangeWithKind::new(10..15, 2, true)).unwrap(); // 与前一个相同 kind

    assert_eq!(set.len(), 2);
    assert_eq!(set.as_slice()[0].range(), &(0..5));
    assert_eq!(set.as_slice()[1].range(), &(5..15)); // 后两个应该合并
}