use ranges_ext::RangeSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut set: RangeSet<i32, 16> = RangeSet::new();
    set.add(1..5)?;
    set.add(3..8)?;
    set.add(10..15)?;
    set.add(12..18)?;

    println!("=== 区间合并结果 ===");
    for (i, range) in set.iter().enumerate() {
        println!("Element {}: [{}, {})", i, range.start, range.end);
    }

    println!("\n=== Debug 格式 ===");
    for (i, range) in set.iter().enumerate() {
        println!("Element {}: {:?}", i, range);
    }

    println!("\n=== 完整切片 ===");
    println!("{:?}", set.as_slice());

    Ok(())
}
