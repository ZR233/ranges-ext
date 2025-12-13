#![cfg(any(windows, unix))]

use core::ops::Range;

use ranges_ext::{CapacityError, RangeSet};

fn r(start: i32, end: i32) -> Range<i32> {
    start..end
}

#[test]
fn add_merges_overlaps_and_adjacency() -> Result<(), CapacityError> {
    let mut set = RangeSet::<i32>::new();
    set.add(r(10, 20), ())?;
    set.add(r(30, 40), ())?;
    set.add(r(15, 35), ())?;

    let expected = [ranges_ext::RangeInfo::new(r(10, 40), ())];
    assert_eq!(set.as_slice(), &expected);

    // 相邻也会合并（[10,20) + [20,25) => [10,25)）
    set.clear();
    set.add(r(10, 20), ())?;
    set.add(r(20, 25), ())?;

    let expected = [ranges_ext::RangeInfo::new(r(10, 25), ())];
    assert_eq!(set.as_slice(), &expected);
    Ok(())
}

#[test]
fn add_out_of_order_is_normalized() -> Result<(), CapacityError> {
    let mut set = RangeSet::<i32>::new();

    // 乱序添加：应当最终排序并正确合并
    set.add(r(30, 40), ())?;
    set.add(r(10, 20), ())?;
    set.add(r(25, 30), ())?;
    set.add(r(20, 25), ())?;

    let expected = [ranges_ext::RangeInfo::new(r(10, 40), ())];
    assert_eq!(set.as_slice(), &expected);

    // 再加一个与头部相交的区间，仍应合并成一个
    set.add(r(0, 12), ())?;

    let expected = [ranges_ext::RangeInfo::new(r(0, 40), ())];
    assert_eq!(set.as_slice(), &expected);
    Ok(())
}

#[test]
fn contains_works() -> Result<(), CapacityError> {
    let mut set = RangeSet::<i32>::new();
    set.extend([(r(10, 20), ()), (r(30, 40), ())])?;

    assert!(set.contains(10));
    assert!(set.contains(19));
    assert!(!set.contains(20));
    assert!(!set.contains(29));
    assert!(set.contains(30));
    assert!(!set.contains(40));
    Ok(())
}

#[test]
fn remove_trims_and_splits() -> Result<(), CapacityError> {
    let mut set = RangeSet::<i32>::new();
    set.add(r(10, 50), ())?;

    // 删除中间，触发分裂
    set.remove(r(20, 30))?;
    let expected = [
        ranges_ext::RangeInfo::new(r(10, 20), ()),
        ranges_ext::RangeInfo::new(r(30, 50), ()),
    ];
    assert_eq!(set.as_slice(), &expected);

    // 删除左侧覆盖
    set.remove(r(0, 12))?;
    let expected = [
        ranges_ext::RangeInfo::new(r(12, 20), ()),
        ranges_ext::RangeInfo::new(r(30, 50), ()),
    ];
    assert_eq!(set.as_slice(), &expected);

    // 删除跨多个区间
    set.remove(r(15, 45))?;
    let expected = [
        ranges_ext::RangeInfo::new(r(12, 15), ()),
        ranges_ext::RangeInfo::new(r(45, 50), ()),
    ];
    assert_eq!(set.as_slice(), &expected);
    Ok(())
}

#[test]
fn remove_noop_on_empty_or_non_overlapping() -> Result<(), CapacityError> {
    let mut set = RangeSet::<i32>::new();
    set.remove(r(1, 2))?;
    assert!(set.is_empty());

    set.add(r(10, 20), ())?;
    set.remove(r(0, 5))?;

    let expected = [ranges_ext::RangeInfo::new(r(10, 20), ())];
    assert_eq!(set.as_slice(), &expected);
    Ok(())
}

#[test]
fn capacity_error_on_overflow() -> Result<(), CapacityError> {
    // 使用容量为 2 的 RangeSet
    let mut set: RangeSet<i32, (), 2> = RangeSet::new();

    // 添加两个不重叠的区间（成功）
    set.add(r(10, 20), ())?;
    set.add(r(30, 40), ())?;

    // 尝试添加第三个区间（应该失败）
    assert_eq!(set.add(r(50, 60), ()), Err(CapacityError));

    Ok(())
}

#[test]
fn only_merge_when_kind_equals() -> Result<(), CapacityError> {
    // 测试只有当 kind 相等时才合并区间
    let mut set = RangeSet::<i32, i32>::new();

    // 添加两个相邻的区间，但 kind 不同，不应合并
    set.add(r(10, 20), 1)?;
    set.add(r(20, 30), 2)?;

    assert_eq!(set.len(), 2);
    let expected = [
        ranges_ext::RangeInfo::new(r(10, 20), 1),
        ranges_ext::RangeInfo::new(r(20, 30), 2),
    ];
    assert_eq!(set.as_slice(), &expected);

    // 添加相邻且 kind 相同的区间，应该合并
    set.add(r(30, 40), 2)?;

    assert_eq!(set.len(), 2);
    let expected = [
        ranges_ext::RangeInfo::new(r(10, 20), 1),
        ranges_ext::RangeInfo::new(r(20, 40), 2),
    ];
    assert_eq!(set.as_slice(), &expected);

    // 添加重叠但 kind 不同的区间，不应合并（会分割）
    set.add(r(15, 25), 3)?;

    assert_eq!(set.len(), 3);
    let expected = [
        ranges_ext::RangeInfo::new(r(10, 15), 1),
        ranges_ext::RangeInfo::new(r(15, 25), 3),
        ranges_ext::RangeInfo::new(r(25, 40), 2),
    ];
    assert_eq!(set.as_slice(), &expected);

    Ok(())
}
