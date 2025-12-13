use ranges_ext::{RangeInfo, RangeSet};
use std::ops::Range;

// 带有字符串 kind 的区间信息实现
#[derive(Clone, Debug, PartialEq, Eq)]
struct StrRange<T> {
    range: core::ops::Range<T>,
    kind: &'static str,
    overwritable: bool,
}

impl<T> StrRange<T> {
    fn new(range: core::ops::Range<T>, kind: &'static str, overwritable: bool) -> Self {
        Self {
            range,
            kind,
            overwritable,
        }
    }
}

impl<T: core::fmt::Debug + Clone + Ord + Copy> RangeInfo for StrRange<T> {
    type Kind = &'static str;
    type Type = T;

    fn range(&self) -> Range<Self::Type> {
        self.range.clone()
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

// 带有整数 kind 的区间信息实现
#[derive(Clone, Debug, PartialEq, Eq)]
struct IntRange<T> {
    range: core::ops::Range<T>,
    kind: i32,
    overwritable: bool,
}

impl<T> IntRange<T> {
    fn new(range: core::ops::Range<T>, kind: i32, overwritable: bool) -> Self {
        Self {
            range,
            kind,
            overwritable,
        }
    }
}

impl<T: core::fmt::Debug + Clone + Ord + Copy> RangeInfo for IntRange<T> {
    type Kind = i32;
    type Type = T;

    fn range(&self) -> Range<Self::Type> {
        self.range.clone()
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用字符串作为 kind 的示例
    let mut set: RangeSet<StrRange<i32>> = RangeSet::new();

    // 添加不同类型的区间（使用不同的 kind）
    set.add(StrRange::new(0..10, "type_a", true))?;
    set.add(StrRange::new(5..15, "type_b", true))?; // 与 type_a 重叠，但 kind 不同，会分割
    set.add(StrRange::new(20..30, "type_a", true))?;
    set.add(StrRange::new(25..35, "type_a", true))?; // 与上一个 type_a 重叠，会合并

    println!("=== 带 kind 的区间集合 ===");
    for info in set.iter() {
        println!(
            "Range: [{}, {}), Kind: {:?}",
            info.range().start,
            info.range().end,
            info.kind()
        );
    }

    // 演示：只有 kind 相同的相邻区间才会合并
    let mut set2: RangeSet<IntRange<i32>> = RangeSet::new();
    set2.add(IntRange::new(0..10, 1, true))?;
    set2.add(IntRange::new(10..20, 1, true))?; // kind 相同且相邻，会合并
    set2.add(IntRange::new(20..30, 2, true))?; // kind 不同，不合并
    set2.add(IntRange::new(30..40, 2, true))?; // kind 相同且相邻，会合并

    println!("\n=== 相邻区间合并示例 ===");
    for info in set2.iter() {
        println!(
            "Range: [{}, {}), Kind: {}",
            info.range().start,
            info.range().end,
            info.kind()
        );
    }

    Ok(())
}
