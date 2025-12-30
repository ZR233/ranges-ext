#![allow(clippy::reversed_empty_ranges)]

mod common;
use common::*;

#[test]
fn test_large_range_operations() {
    // 测试大范围的操作，确保没有整数溢出
    let mut set = RangeSetHeapless::<TestRange<u32>>::default();

    // 使用接近 u32::MAX 的值
    let max_val = u32::MAX - 10;
    set.add(TestRange::new(max_val..u32::MAX, true)).unwrap();
    assert_eq!(set.len(), 1);

    // 测试包含检查
    assert!(set.contains(max_val));
    assert!(set.contains(u32::MAX - 1));
    assert!(!set.contains(u32::MAX));
}

#[test]
fn test_remove_empty_set() {
    // 测试对空集合的删除操作
    let mut set = RangeSetHeapless::<TestRange<i32>>::default();

    // 删除空集合应该 no-op
    let result = set.remove(0..10);
    assert!(result.is_ok());
    assert!(set.is_empty());

    // 删除反向区间也应该 no-op
    let result = set.remove(10..0);
    assert!(result.is_ok());
    assert!(set.is_empty());
}

#[test]
fn test_identical_ranges() {
    // 测试添加完全相同的区间
    let mut set = RangeSetHeapless::<TestRangeWithKind<i32, i32>>::default();

    set.add(TestRangeWithKind::new(0..10, 1, true)).unwrap();
    assert_eq!(set.len(), 1);

    // 添加相同的区间（应该合并）
    set.add(TestRangeWithKind::new(0..10, 1, true)).unwrap();
    assert_eq!(set.len(), 1);
    assert_eq!(set.as_slice()[0].range(), (0..10));

    // 添加相同区间但不同 kind（应该替换）
    set.add(TestRangeWithKind::new(0..10, 2, true)).unwrap();
    assert_eq!(set.len(), 1);
    assert_eq!(set.as_slice()[0].range(), (0..10));
    assert_eq!(set.as_slice()[0].kind(), 2);
}

#[test]
fn test_extend_with_errors() {
    // 测试 extend 方法遇到错误时的行为
    let mut set: RangeSetHeapless<TestRange<i32>, 2> = RangeSetHeapless::default();

    set.add(TestRange::new(0..5, true)).unwrap();

    // extend 包含多个区间，其中一个会导致错误
    let ranges = [
        TestRange::new(10..15, true), // OK
        TestRange::new(20..25, true), // 会导致容量溢出
    ];

    let result = set.extend(ranges);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), RangeError::Capacity);

    // extend 在遇到错误前已经成功添加了第一个区间
    assert_eq!(set.len(), 2);

    // 但是第三个区间不会被添加
    assert!(!set.iter().any(|r| r.range().start == 20));
}

#[test]
fn test_iteration_order() {
    // 测试迭代器返回的区间是否按顺序排列
    let mut set = RangeSetHeapless::<TestRange<i32>>::default();

    // 乱序添加
    set.add(TestRange::new(30..40, true)).unwrap();
    set.add(TestRange::new(10..20, true)).unwrap();
    set.add(TestRange::new(0..5, true)).unwrap();

    // 验证顺序
    let mut last_end = None;
    for info in set.iter() {
        if let Some(end) = last_end {
            assert!(info.range().start >= end);
        }
        last_end = Some(info.range().end);
    }
}

#[test]
fn test_clear_and_reuse() {
    // 测试 clear 后重新使用
    let mut set = RangeSetHeapless::<TestRangeWithKind<i32, i32>>::default();

    set.add(TestRangeWithKind::new(0..10, 1, true)).unwrap();
    set.add(TestRangeWithKind::new(20..30, 2, true)).unwrap();
    assert_eq!(set.len(), 2);

    set.clear();
    assert!(set.is_empty());

    // 重新使用
    set.add(TestRangeWithKind::new(5..15, 3, true)).unwrap();
    assert_eq!(set.len(), 1);
    assert_eq!(set.as_slice()[0].range(), (5..15));
}
