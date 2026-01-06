#![cfg(feature = "alloc")]
#![cfg(any(windows, unix))]

extern crate alloc;

mod common;
use alloc::vec::Vec;
use common::*;

fn r(start: i32, end: i32) -> core::ops::Range<i32> {
    start..end
}

#[test]
fn alloc_add_merges_overlaps_and_adjacency() {
    let mut set = Vec::<TestRange<i32>>::new();
    set.test_add(TestRange::new(r(10, 20), true)).unwrap();
    set.test_add(TestRange::new(r(30, 40), true)).unwrap();
    set.test_add(TestRange::new(r(15, 35), true)).unwrap();

    let expected = [TestRange::new(r(10, 40), true)];
    assert_eq!(set.as_slice(), &expected);

    // 相邻也会合并（[10,20) + [20,25) => [10,25)）
    set.clear();
    set.test_add(TestRange::new(r(10, 20), true)).unwrap();
    set.test_add(TestRange::new(r(20, 25), true)).unwrap();

    let expected = [TestRange::new(r(10, 25), true)];
    assert_eq!(set.as_slice(), &expected);
}

#[test]
fn alloc_add_out_of_order_is_normalized() {
    let mut set = Vec::<TestRange<i32>>::new();

    // 乱序添加：应当最终排序并正确合并
    set.test_add(TestRange::new(r(30, 40), true)).unwrap();
    set.test_add(TestRange::new(r(10, 20), true)).unwrap();
    set.test_add(TestRange::new(r(25, 30), true)).unwrap();
    set.test_add(TestRange::new(r(20, 25), true)).unwrap();

    let expected = [TestRange::new(r(10, 40), true)];
    assert_eq!(set.as_slice(), &expected);

    // 再加一个与头部相交的区间，仍应合并成一个
    set.test_add(TestRange::new(r(0, 12), true)).unwrap();

    let expected = [TestRange::new(r(0, 40), true)];
    assert_eq!(set.as_slice(), &expected);
}

#[test]
fn alloc_contains_works() {
    let mut set = Vec::<TestRange<i32>>::new();
    set.test_extend([
        TestRange::new(r(10, 20), true),
        TestRange::new(r(30, 40), true),
    ])
    .unwrap();

    assert!(set.test_contains_point(10));
    assert!(set.test_contains_point(19));
    assert!(!set.test_contains_point(20));
    assert!(!set.test_contains_point(29));
    assert!(set.test_contains_point(30));
    assert!(!set.test_contains_point(40));
}

#[test]
fn alloc_remove_trims_and_splits() {
    let mut set = Vec::<TestRange<i32>>::new();
    set.test_add(TestRange::new(r(10, 50), true)).unwrap();

    // 删除中间，触发分裂
    set.test_remove(r(20, 30)).unwrap();
    let expected = [
        TestRange::new(r(10, 20), true),
        TestRange::new(r(30, 50), true),
    ];
    assert_eq!(set.as_slice(), &expected);

    // 删除左侧覆盖
    set.test_remove(r(0, 12)).unwrap();
    let expected = [
        TestRange::new(r(12, 20), true),
        TestRange::new(r(30, 50), true),
    ];
    assert_eq!(set.as_slice(), &expected);

    // 删除跨多个区间
    set.test_remove(r(15, 45)).unwrap();
    let expected = [
        TestRange::new(r(12, 15), true),
        TestRange::new(r(45, 50), true),
    ];
    assert_eq!(set.as_slice(), &expected);
}

#[test]
fn alloc_remove_noop_on_empty_or_non_overlapping() {
    let mut set = Vec::<TestRange<i32>>::new();
    set.test_remove(r(1, 2)).unwrap();
    assert!(set.is_empty());

    set.test_add(TestRange::new(r(10, 20), true)).unwrap();
    set.test_remove(r(0, 10)).unwrap();
    let expected = [TestRange::new(r(10, 20), true)];
    assert_eq!(set.as_slice(), &expected);
}

#[test]
fn alloc_multiple_non_overlapping() {
    let mut set = Vec::<TestRange<i32>>::new();
    set.test_add(TestRange::new(r(10, 20), true)).unwrap();
    set.test_add(TestRange::new(r(30, 40), true)).unwrap();
    set.test_add(TestRange::new(r(50, 60), true)).unwrap();

    let expected = [
        TestRange::new(r(10, 20), true),
        TestRange::new(r(30, 40), true),
        TestRange::new(r(50, 60), true),
    ];
    assert_eq!(set.as_slice(), &expected);
}

#[test]
fn alloc_extend_merges_correctly() {
    let mut set = Vec::<TestRange<i32>>::new();
    set.test_extend([
        TestRange::new(r(10, 20), true),
        TestRange::new(r(15, 25), true),
        TestRange::new(r(30, 40), true),
    ])
    .unwrap();

    let expected = [
        TestRange::new(r(10, 25), true),
        TestRange::new(r(30, 40), true),
    ];
    assert_eq!(set.as_slice(), &expected);
}

#[test]
fn alloc_remove_entire_range() {
    let mut set = Vec::<TestRange<i32>>::new();
    set.test_add(TestRange::new(r(10, 50), true)).unwrap();
    set.test_remove(r(10, 50)).unwrap();
    assert!(set.is_empty());
}

#[test]
fn alloc_remove_partial_left() {
    let mut set = Vec::<TestRange<i32>>::new();
    set.test_add(TestRange::new(r(10, 50), true)).unwrap();
    set.test_remove(r(10, 30)).unwrap();

    let expected = [TestRange::new(r(30, 50), true)];
    assert_eq!(set.as_slice(), &expected);
}

#[test]
fn alloc_remove_partial_right() {
    let mut set = Vec::<TestRange<i32>>::new();
    set.test_add(TestRange::new(r(10, 50), true)).unwrap();
    set.test_remove(r(30, 50)).unwrap();

    let expected = [TestRange::new(r(10, 30), true)];
    assert_eq!(set.as_slice(), &expected);
}

#[test]
fn alloc_large_scale_operations() {
    let mut set = Vec::<TestRange<i32>>::new();

    // 添加多个不重叠的区间
    for i in 0..10 {
        set.test_add(TestRange::new(r(i * 100, i * 100 + 50), true))
            .unwrap();
    }
    assert_eq!(set.len(), 10);

    // 添加覆盖所有区间的大区间
    set.test_add(TestRange::new(r(0, 1000), true)).unwrap();
    assert_eq!(set.len(), 1);
    assert_eq!(set[0].range, r(0, 1000));
}

#[test]
fn alloc_boundary_conditions() {
    let mut set = Vec::<TestRange<i32>>::new();

    // 测试空区间
    set.test_add(TestRange::new(r(10, 10), true)).unwrap();
    assert!(set.is_empty());

    // 测试单点区间
    set.test_add(TestRange::new(r(10, 11), true)).unwrap();
    let expected = [TestRange::new(r(10, 11), true)];
    assert_eq!(set.as_slice(), &expected);

    // 测试相邻区间合并
    set.test_add(TestRange::new(r(11, 12), true)).unwrap();
    let expected = [TestRange::new(r(10, 12), true)];
    assert_eq!(set.as_slice(), &expected);
}

#[test]
fn alloc_negative_ranges() {
    let mut set = Vec::<TestRange<i32>>::new();
    set.test_add(TestRange::new(r(-50, -10), true)).unwrap();
    set.test_add(TestRange::new(r(-20, 0), true)).unwrap();

    let expected = [TestRange::new(r(-50, 0), true)];
    assert_eq!(set.as_slice(), &expected);

    assert!(set.test_contains_point(-25));
    assert!(!set.test_contains_point(-60));
    assert!(!set.test_contains_point(10));
}
