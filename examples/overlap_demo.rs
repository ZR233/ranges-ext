use ranges_ext::{RangeInfo, RangeSetHeapless};
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

    fn kind(&self) -> Self::Kind {
        self.kind
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
    println!("=== 测试：kind 不同时，后者覆盖交集 ===\n");

    let mut set = RangeSetHeapless::<StrRange<i32>>::default();

    // 场景 1: 完全覆盖
    println!("场景 1: 完全覆盖");
    set.add(StrRange::new(10..20, "A", true))?;
    println!("添加 [10, 20) kind=A");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range().start,
            info.range().end,
            info.kind()
        );
    }

    set.add(StrRange::new(10..20, "B", true))?;
    println!("添加 [10, 20) kind=B (完全覆盖)");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range().start,
            info.range().end,
            info.kind()
        );
    }

    // 场景 2: 部分覆盖（左侧）
    println!("\n场景 2: 部分覆盖（左侧）");
    set.clear();
    set.add(StrRange::new(10..30, "A", true))?;
    println!("添加 [10, 30) kind=A");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range().start,
            info.range().end,
            info.kind()
        );
    }

    set.add(StrRange::new(5..20, "B", true))?;
    println!("添加 [5, 20) kind=B (覆盖左侧)");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range().start,
            info.range().end,
            info.kind()
        );
    }

    // 场景 3: 部分覆盖（右侧）
    println!("\n场景 3: 部分覆盖（右侧）");
    set.clear();
    set.add(StrRange::new(10..30, "A", true))?;
    println!("添加 [10, 30) kind=A");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range().start,
            info.range().end,
            info.kind()
        );
    }

    set.add(StrRange::new(20..35, "B", true))?;
    println!("添加 [20, 35) kind=B (覆盖右侧)");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range().start,
            info.range().end,
            info.kind()
        );
    }

    // 场景 4: 中间覆盖（分裂原区间）
    println!("\n场景 4: 中间覆盖（分裂原区间）");
    set.clear();
    set.add(StrRange::new(10..40, "A", true))?;
    println!("添加 [10, 40) kind=A");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range().start,
            info.range().end,
            info.kind()
        );
    }

    set.add(StrRange::new(20..30, "B", true))?;
    println!("添加 [20, 30) kind=B (覆盖中间，分裂 A)");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range().start,
            info.range().end,
            info.kind()
        );
    }

    // 场景 5: kind 相同时合并
    println!("\n场景 5: kind 相同时合并（对比）");
    set.clear();
    set.add(StrRange::new(10..20, "A", true))?;
    set.add(StrRange::new(30..40, "A", true))?;
    println!("添加 [10, 20) kind=A 和 [30, 40) kind=A");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range().start,
            info.range().end,
            info.kind()
        );
    }

    set.add(StrRange::new(15..35, "A", true))?;
    println!("添加 [15, 35) kind=A (相同 kind，合并)");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range().start,
            info.range().end,
            info.kind()
        );
    }

    // 场景 6: 复杂场景 - 多个区间
    println!("\n场景 6: 复杂场景 - 覆盖多个不同 kind 的区间");
    set.clear();
    set.add(StrRange::new(0..10, "A", true))?;
    set.add(StrRange::new(10..20, "B", true))?;
    set.add(StrRange::new(20..30, "C", true))?;
    println!("初始状态:");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range().start,
            info.range().end,
            info.kind()
        );
    }

    set.add(StrRange::new(5..25, "D", true))?;
    println!("添加 [5, 25) kind=D (覆盖三个区间的部分)");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range().start,
            info.range().end,
            info.kind()
        );
    }

    Ok(())
}
