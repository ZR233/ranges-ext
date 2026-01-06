use ranges_ext::{RangeInfo, RangeSetOps};
use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct DemoRange<T> {
    range: Range<T>,
    kind: &'static str,
    overwritable: bool,
}

impl<T> DemoRange<T> {
    fn new(range: Range<T>, kind: &'static str, overwritable: bool) -> Self {
        Self {
            range,
            kind,
            overwritable,
        }
    }
}

impl<T: core::fmt::Debug + Clone + Ord + Copy + Default> RangeInfo for DemoRange<T> {
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
    println!("=== 使用字节缓冲区演示 ===\n");

    // 创建一个容量较小的 heapless::Vec
    let mut set: heapless::Vec<DemoRange<i32>, 8> = heapless::Vec::new();

    // 创建字节缓冲区用于 add/remove 操作
    // 大小计算：size_of::<DemoRange<i32>>() * 预期容量
    // DemoRange<i32> 大约 24 字节，预留 24 个元素的空间
    let mut temp_buffer = [0u8; 24 * 24];

    println!("场景 1: 使用字节缓冲区添加需要分割的区间");
    set.merge_add(DemoRange::new(0..10, "A", true), &mut temp_buffer)?;
    set.merge_add(DemoRange::new(20..30, "A", true), &mut temp_buffer)?;
    set.merge_add(DemoRange::new(40..50, "B", true), &mut temp_buffer)?;

    println!("初始状态:");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={} overwritable={}",
            info.range().start,
            info.range().end,
            info.kind(),
            info.overwritable()
        );
    }

    // 添加一个会导致分割的区间
    println!("\n添加 [5, 45) kind=C (会覆盖和分割多个区间):");
    set.merge_add(DemoRange::new(5..45, "C", false), &mut temp_buffer)?;

    for info in set.iter() {
        println!(
            "  [{}, {}) kind={} overwritable={}",
            info.range().start,
            info.range().end,
            info.kind(),
            info.overwritable()
        );
    }

    println!("\n场景 2: 使用字节缓冲区删除区间");
    println!("删除 [10, 35):");
    set.merge_remove(10..35, &mut temp_buffer)?;

    for info in set.iter() {
        println!(
            "  [{}, {}) kind={} overwritable={}",
            info.range().start,
            info.range().end,
            info.kind(),
            info.overwritable()
        );
    }

    println!("\n场景 3: 字节缓冲区可以复用");
    println!("继续添加新区间使用同一个字节缓冲区:");
    set.merge_add(DemoRange::new(50..60, "D", true), &mut temp_buffer)?;
    set.merge_add(DemoRange::new(55..65, "D", true), &mut temp_buffer)?;

    println!("最终状态:");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={} overwritable={}",
            info.range().start,
            info.range().end,
            info.kind(),
            info.overwritable()
        );
    }

    println!("\n场景 4: 在栈上分配字节缓冲区（no_std 友好）");
    println!("使用 &mut [u8] 作为临时缓冲区的优势：");
    println!("  - 可以在栈上分配固定大小的字节数组");
    println!("  - 无需堆分配，适合嵌入式或 no_std 环境");
    println!("  - 可以使用静态分配的全局缓冲区");
    println!("  - 对于不同大小的 T，只需调整字节数组大小");

    Ok(())
}
