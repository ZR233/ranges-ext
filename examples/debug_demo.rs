use ranges_ext::{RangeInfo, RangeSet};
use std::ops::Range;

// 简单的区间信息实现，用于示例
#[derive(Clone, Debug, PartialEq, Eq)]
struct DemoRange<T> {
    range: core::ops::Range<T>,
    kind: (),
    overwritable: bool,
}

impl<T> DemoRange<T> {
    fn new(range: core::ops::Range<T>, overwritable: bool) -> Self {
        Self {
            range,
            kind: (),
            overwritable,
        }
    }
}

impl<T: core::fmt::Debug + Clone + Ord + Copy> RangeInfo for DemoRange<T> {
    type Kind = ();
    type Type = T;

    fn range(&self) -> &Range<Self::Type> {
        &self.range
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
    let mut set: RangeSet<DemoRange<i32>, 16> = RangeSet::new();
    set.add(DemoRange::new(1..5, true))?;
    set.add(DemoRange::new(3..8, true))?;
    set.add(DemoRange::new(10..15, true))?;
    set.add(DemoRange::new(12..18, true))?;

    println!("=== 区间合并结果 ===");
    for (i, info) in set.iter().enumerate() {
        println!(
            "Element {}: [{}, {})",
            i,
            info.range().start,
            info.range().end
        );
    }

    println!("\n=== Debug 格式 ===");
    for (i, info) in set.iter().enumerate() {
        println!("Element {}: {:?}", i, info);
    }

    println!("\n=== 完整切片 ===");
    println!("{:?}", set.as_slice());

    Ok(())
}
