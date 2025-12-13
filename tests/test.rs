#![cfg(any(windows, unix))]

use core::ops::Range;

use ranges_ext::RangeSet;

fn r(start: i32, end: i32) -> Range<i32> {
    start..end
}

// 辅助函数：将 heapless::Vec 转换为标准 Vec 进行比较
fn assert_ranges_eq<const C: usize>(
    actual: heapless::Vec<Range<i32>, C>,
    expected: Vec<Range<i32>>,
) {
    assert_eq!(actual.len(), expected.len(), "长度不匹配");
    for (a, e) in actual.iter().zip(expected.iter()) {
        assert_eq!(a, e);
    }
}

#[test]
fn add_merges_overlaps_and_adjacency() {
    let mut set = RangeSet::<i32>::new();
    set.add_range(r(10, 20));
    set.add_range(r(30, 40));
    set.add_range(r(15, 35));
    assert_ranges_eq(set.as_slice(), vec![r(10, 40)]);

    // 相邻也会合并（[10,20) + [20,25) => [10,25)）
    set.clear();
    set.add_range(r(10, 20));
    set.add_range(r(20, 25));
    assert_ranges_eq(set.as_slice(), vec![r(10, 25)]);
}

#[test]
fn add_out_of_order_is_normalized() {
    let mut set = RangeSet::<i32>::new();

    // 乱序添加：应当最终排序并正确合并
    set.add_range(r(30, 40));
    set.add_range(r(10, 20));
    set.add_range(r(25, 30));
    set.add_range(r(20, 25));
    assert_ranges_eq(set.as_slice(), vec![r(10, 40)]);

    // 再加一个与头部相交的区间，仍应合并成一个
    set.add_range(r(0, 12));
    assert_ranges_eq(set.as_slice(), vec![r(0, 40)]);
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
    set.add_range(r(10, 50));

    // 删除中间，触发分裂
    set.remove_range(r(20, 30));
    assert_ranges_eq(set.as_slice(), vec![r(10, 20), r(30, 50)]);

    // 删除左侧覆盖
    set.remove_range(r(0, 12));
    assert_ranges_eq(set.as_slice(), vec![r(12, 20), r(30, 50)]);

    // 删除跨多个区间
    set.remove_range(r(15, 45));
    assert_ranges_eq(set.as_slice(), vec![r(12, 15), r(45, 50)]);
}

#[test]
fn remove_noop_on_empty_or_non_overlapping() {
    let mut set = RangeSet::<i32>::new();
    set.remove_range(r(1, 2));
    assert!(set.is_empty());

    set.add_range(r(10, 20));
    set.remove_range(r(0, 5));
    assert_ranges_eq(set.as_slice(), vec![r(10, 20)]);
}

#[test]
fn metadata_preserved_and_removed() {
    let mut set = RangeSet::<i32, &str>::new();

    // 添加三个区间，前两个会合并
    set.add(r(10, 20), "first");
    set.add(r(15, 25), "second");
    set.add(r(30, 40), "third");

    // 验证合并：[10,25) 包含两个原始区间
    assert_ranges_eq(set.as_slice(), vec![r(10, 25), r(30, 40)]);
    assert_eq!(set.elements().len(), 2);
    assert_eq!(set.elements()[0].originals.len(), 2);
    assert_eq!(set.elements()[1].originals.len(), 1);

    // 删除完全包含 "first" 的区间 [10,20)
    set.remove_range(r(10, 20));

    // 验证：合并区间被分裂，"first" 被移除，"second" 保留
    assert_ranges_eq(set.as_slice(), vec![r(20, 25), r(30, 40)]);
    let remaining = &set.elements()[0].originals;
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].meta, "second");

    // 删除 "third" 的一部分 - 不完全包含，所以保留
    set.remove_range(r(32, 35));
    assert_ranges_eq(set.as_slice(), vec![r(20, 25), r(30, 32), r(35, 40)]);
    // "third" 仍在两个分裂的片段中
    assert_eq!(set.elements()[1].originals[0].meta, "third");
    assert_eq!(set.elements()[2].originals[0].meta, "third");
}

#[test]
fn originals_merge_when_metadata_equal_and_adjacent() {
    let mut set = RangeSet::<i32, &str>::new();

    // 添加相同 metadata 的相邻区间
    set.add(r(10, 20), "same");
    set.add(r(20, 30), "same");
    set.add(r(30, 40), "same");

    // 合并后应该只有一个区间，原始列表中也应该合并成一个
    assert_ranges_eq(set.as_slice(), vec![r(10, 40)]);
    assert_eq!(set.elements().len(), 1);
    assert_eq!(set.elements()[0].originals.len(), 1);
    assert_eq!(set.elements()[0].originals[0].range, r(10, 40));
    assert_eq!(set.elements()[0].originals[0].meta, "same");

    // 添加不同 metadata 的相邻区间
    set.add(r(40, 50), "different");

    // 合并后区间连续，但原始列表保留两个（因为 metadata 不同）
    assert_ranges_eq(set.as_slice(), vec![r(10, 50)]);
    assert_eq!(set.elements()[0].originals.len(), 2);
    assert_eq!(set.elements()[0].originals[0].range, r(10, 40));
    assert_eq!(set.elements()[0].originals[0].meta, "same");
    assert_eq!(set.elements()[0].originals[1].range, r(40, 50));
    assert_eq!(set.elements()[0].originals[1].meta, "different");
}

#[test]
fn originals_merge_overlapping_same_metadata() {
    let mut set = RangeSet::<i32, i32>::new();

    // 添加相同 metadata 的重叠区间
    set.add(r(10, 25), 100);
    set.add(r(20, 35), 100);
    set.add(r(30, 40), 100);

    // 原始列表应该合并成一个
    assert_ranges_eq(set.as_slice(), vec![r(10, 40)]);
    assert_eq!(set.elements()[0].originals.len(), 1);
    assert_eq!(set.elements()[0].originals[0].range, r(10, 40));
    assert_eq!(set.elements()[0].originals[0].meta, 100);
}
