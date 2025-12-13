use ranges_ext::RangeSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用字符串作为 kind 的示例
    let mut set: RangeSet<i32, &str> = RangeSet::new();

    // 添加不同类型的区间（使用不同的 kind）
    set.add(0..10, "type_a", true)?;
    set.add(5..15, "type_b", true)?; // 与 type_a 重叠，但 kind 不同，会分割
    set.add(20..30, "type_a", true)?;
    set.add(25..35, "type_a", true)?; // 与上一个 type_a 重叠，会合并

    println!("=== 带 kind 的区间集合 ===");
    for info in set.iter() {
        println!(
            "Range: [{}, {}), Kind: {:?}",
            info.range.start, info.range.end, info.kind
        );
    }

    // 演示：只有 kind 相同的相邻区间才会合并
    let mut set2: RangeSet<i32, i32> = RangeSet::new();
    set2.add(0..10, 1, true)?;
    set2.add(10..20, 1, true)?; // kind 相同且相邻，会合并
    set2.add(20..30, 2, true)?; // kind 不同，不合并
    set2.add(30..40, 2, true)?; // kind 相同且相邻，会合并

    println!("\n=== 相邻区间合并示例 ===");
    for info in set2.iter() {
        println!(
            "Range: [{}, {}), Kind: {}",
            info.range.start, info.range.end, info.kind
        );
    }

    Ok(())
}
