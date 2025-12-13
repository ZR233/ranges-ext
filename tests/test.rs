#![cfg(any(windows, unix))]

use core::ops::Range;

use ranges_ext::{CapacityError, RangeSet};

fn r(start: i32, end: i32) -> Range<i32> {
    start..end
}

#[test]
fn add_merges_overlaps_and_adjacency() {
    let mut set = RangeSet::<i32>::new();
    set.add(r(10, 20), (), true).unwrap();
    set.add(r(30, 40), (), true).unwrap();
    set.add(r(15, 35), (), true).unwrap();

    let expected = [ranges_ext::OneRange::new(r(10, 40), (), true)];
    assert_eq!(set.as_slice(), &expected);

    // 相邻也会合并（[10,20) + [20,25) => [10,25)）
    set.clear();
    set.add(r(10, 20), (), true).unwrap();
    set.add(r(20, 25), (), true).unwrap();

    let expected = [ranges_ext::OneRange::new(r(10, 25), (), true)];
    assert_eq!(set.as_slice(), &expected);
}

#[test]
fn add_out_of_order_is_normalized() {
    let mut set = RangeSet::<i32>::new();

    // 乱序添加：应当最终排序并正确合并
    set.add(r(30, 40), (), true).unwrap();
    set.add(r(10, 20), (), true).unwrap();
    set.add(r(25, 30), (), true).unwrap();
    set.add(r(20, 25), (), true).unwrap();

    let expected = [ranges_ext::OneRange::new(r(10, 40), (), true)];
    assert_eq!(set.as_slice(), &expected);

    // 再加一个与头部相交的区间，仍应合并成一个
    set.add(r(0, 12), (), true).unwrap();

    let expected = [ranges_ext::OneRange::new(r(0, 40), (), true)];
    assert_eq!(set.as_slice(), &expected);
}

#[test]
fn contains_works() {
    let mut set = RangeSet::<i32>::new();
    set.extend([(r(10, 20), (), true), (r(30, 40), (), true)])
        .unwrap();

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
    set.add(r(10, 50), (), true).unwrap();

    // 删除中间，触发分裂
    set.remove(r(20, 30)).unwrap();
    let expected = [
        ranges_ext::OneRange::new(r(10, 20), (), true),
        ranges_ext::OneRange::new(r(30, 50), (), true),
    ];
    assert_eq!(set.as_slice(), &expected);

    // 删除左侧覆盖
    set.remove(r(0, 12)).unwrap();
    let expected = [
        ranges_ext::OneRange::new(r(12, 20), (), true),
        ranges_ext::OneRange::new(r(30, 50), (), true),
    ];
    assert_eq!(set.as_slice(), &expected);

    // 删除跨多个区间
    set.remove(r(15, 45)).unwrap();
    let expected = [
        ranges_ext::OneRange::new(r(12, 15), (), true),
        ranges_ext::OneRange::new(r(45, 50), (), true),
    ];
    assert_eq!(set.as_slice(), &expected);
}

#[test]
fn remove_noop_on_empty_or_non_overlapping() {
    let mut set = RangeSet::<i32>::new();
    set.remove(r(1, 2)).unwrap();
    assert!(set.is_empty());

    set.add(r(10, 20), (), true).unwrap();
    set.remove(r(0, 5)).unwrap();

    let expected = [ranges_ext::OneRange::new(r(10, 20), (), true)];
    assert_eq!(set.as_slice(), &expected);
}

#[test]
fn capacity_error_on_overflow() {
    // 使用容量为 2 的 RangeSet
    let mut set: RangeSet<i32, (), 2> = RangeSet::new();

    // 添加两个不重叠的区间（成功）
    set.add(r(10, 20), (), true).unwrap();
    set.add(r(30, 40), (), true).unwrap();

    // 尝试添加第三个区间（应该失败）
    assert_eq!(set.add(r(50, 60), (), true), Err(Ok(CapacityError)));
}

#[test]
fn only_merge_when_kind_equals() {
    // 测试只有当 kind 相等时才合并区间
    let mut set = RangeSet::<i32, i32>::new();

    // 添加两个相邻的区间，但 kind 不同，不应合并
    set.add(r(10, 20), 1, true).unwrap();
    set.add(r(20, 30), 2, true).unwrap();

    assert_eq!(set.len(), 2);
    let expected = [
        ranges_ext::OneRange::new(r(10, 20), 1, true),
        ranges_ext::OneRange::new(r(20, 30), 2, true),
    ];
    assert_eq!(set.as_slice(), &expected);

    // 添加相邻且 kind 相同的区间，应该合并
    set.add(r(30, 40), 2, true).unwrap();

    assert_eq!(set.len(), 2);
    let expected = [
        ranges_ext::OneRange::new(r(10, 20), 1, true),
        ranges_ext::OneRange::new(r(20, 40), 2, true),
    ];
    assert_eq!(set.as_slice(), &expected);

    // 添加重叠但 kind 不同的区间，不应合并（会分割）
    set.add(r(15, 25), 3, true).unwrap();

    assert_eq!(set.len(), 3);
    let expected = [
        ranges_ext::OneRange::new(r(10, 15), 1, true),
        ranges_ext::OneRange::new(r(15, 25), 3, true),
        ranges_ext::OneRange::new(r(25, 40), 2, true),
    ];
    assert_eq!(set.as_slice(), &expected);
}

#[test]
fn conflict_error_on_non_overwritable() {
    use ranges_ext::ConflictError;

    let mut set = RangeSet::<i32, i32>::new();

    // 添加一个不可覆盖的区间
    set.add(r(10, 30), 1, false).unwrap();

    // 尝试添加一个与之冲突的区间（应该失败）
    let result = set.add(r(20, 40), 2, true);
    assert!(result.is_err());

    if let Err(Err(conflict)) = result {
        assert_eq!(conflict.new.range, r(20, 40));
        assert_eq!(conflict.new.kind, 2);
        assert_eq!(conflict.existing.range, r(10, 30));
        assert_eq!(conflict.existing.kind, 1);
    } else {
        panic!("Expected ConflictError");
    }

    // 验证原区间未被修改
    assert_eq!(set.len(), 1);
    let expected = [ranges_ext::OneRange::new(r(10, 30), 1, false)];
    assert_eq!(set.as_slice(), &expected);
}

#[test]
fn overwritable_ranges_can_be_replaced() {
    let mut set = RangeSet::<i32, i32>::new();

    // 添加一个可覆盖的区间
    set.add(r(10, 30), 1, true).unwrap();

    // 添加一个与之冲突的区间（应该成功，因为旧区间可覆盖）
    set.add(r(20, 40), 2, false).unwrap();

    // 验证结果：旧区间被分割，新区间插入
    assert_eq!(set.len(), 2);
    let expected = [
        ranges_ext::OneRange::new(r(10, 20), 1, true),
        ranges_ext::OneRange::new(r(20, 40), 2, false),
    ];
    assert_eq!(set.as_slice(), &expected);
}
