use ranges_ext::RangeSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut set: RangeSet<i32, (), 16> = RangeSet::new();
    set.add(1..5, (), true).map_err(|e| e.map_err(|_| "conflict").unwrap_err())?;
    set.add(3..8, (), true).map_err(|e| e.map_err(|_| "conflict").unwrap_err())?;
    set.add(10..15, (), true).map_err(|e| e.map_err(|_| "conflict").unwrap_err())?;
    set.add(12..18, (), true).map_err(|e| e.map_err(|_| "conflict").unwrap_err())?;

    println!("=== 区间合并结果 ===");
    for (i, info) in set.iter().enumerate() {
        println!("Element {}: [{}, {})", i, info.range.start, info.range.end);
    }

    println!("\n=== Debug 格式 ===");
    for (i, info) in set.iter().enumerate() {
        println!("Element {}: {:?}", i, info);
    }

    println!("\n=== 完整切片 ===");
    println!("{:?}", set.as_slice());

    Ok(())
}
