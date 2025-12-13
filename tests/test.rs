#![cfg(any(windows, unix))]

use core::ops::Range;

use ranges_ext::RangeSet;

fn r(start: i32, end: i32) -> Range<i32> {
    start..end
}

#[test]
fn add_merges_overlaps_and_adjacency() {
    let mut set = RangeSet::<i32>::new();
    set.add(r(10, 20));
    set.add(r(30, 40));
    set.add(r(15, 35));
    assert_eq!(set.as_slice(), &[r(10, 40)]);

    // 相邻也会合并（[10,20) + [20,25) => [10,25)）
    set.clear();
    set.add(r(10, 20));
    set.add(r(20, 25));
    assert_eq!(set.as_slice(), &[r(10, 25)]);
}

#[test]
fn add_out_of_order_is_normalized() {
    let mut set = RangeSet::<i32>::new();

    // 乱序添加：应当最终排序并正确合并
    set.add(r(30, 40));
    set.add(r(10, 20));
    set.add(r(25, 30));
    set.add(r(20, 25));
    assert_eq!(set.as_slice(), &[r(10, 40)]);

    // 再加一个与头部相交的区间，仍应合并成一个
    set.add(r(0, 12));
    assert_eq!(set.as_slice(), &[r(0, 40)]);
}

#[test]
fn contains_works() {
    let mut set = RangeSet::<i32>::new();
    set.extend([r(10, 20), r(30, 40)]);

    assert!(set.contains(10));
    assert!(set.contains(19));
    assert!(!set.contains(20));
    assert!(!set.contains(29));
    assert!(set.contains(30));
    assert!(!set.contains(40));
}

#[test]
fn remove_trims_and_splits() {
    let mut set = RangeSet::<i32>::new();
    set.add(r(10, 50));

    // 删除中间，触发分裂
    set.remove(r(20, 30));
    assert_eq!(set.as_slice(), &[r(10, 20), r(30, 50)]);

    // 删除左侧覆盖
    set.remove(r(0, 12));
    assert_eq!(set.as_slice(), &[r(12, 20), r(30, 50)]);

    // 删除跨多个区间
    set.remove(r(15, 45));
    assert_eq!(set.as_slice(), &[r(12, 15), r(45, 50)]);
}

#[test]
fn remove_noop_on_empty_or_non_overlapping() {
    let mut set = RangeSet::<i32>::new();
    set.remove(r(1, 2));
    assert!(set.is_empty());

    set.add(r(10, 20));
    set.remove(r(0, 5));
    assert_eq!(set.as_slice(), &[r(10, 20)]);
}
