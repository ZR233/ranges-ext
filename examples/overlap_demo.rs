use ranges_ext::RangeSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 测试：kind 不同时，后者覆盖交集 ===\n");

    let mut set = RangeSet::<i32, &str>::new();

    // 场景 1: 完全覆盖
    println!("场景 1: 完全覆盖");
    set.add(10..20, "A")?;
    println!("添加 [10, 20) kind=A");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range.start, info.range.end, info.kind
        );
    }

    set.add(10..20, "B")?;
    println!("添加 [10, 20) kind=B (完全覆盖)");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range.start, info.range.end, info.kind
        );
    }

    // 场景 2: 部分覆盖（左侧）
    println!("\n场景 2: 部分覆盖（左侧）");
    set.clear();
    set.add(10..30, "A")?;
    println!("添加 [10, 30) kind=A");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range.start, info.range.end, info.kind
        );
    }

    set.add(5..20, "B")?;
    println!("添加 [5, 20) kind=B (覆盖左侧)");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range.start, info.range.end, info.kind
        );
    }

    // 场景 3: 部分覆盖（右侧）
    println!("\n场景 3: 部分覆盖（右侧）");
    set.clear();
    set.add(10..30, "A")?;
    println!("添加 [10, 30) kind=A");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range.start, info.range.end, info.kind
        );
    }

    set.add(20..35, "B")?;
    println!("添加 [20, 35) kind=B (覆盖右侧)");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range.start, info.range.end, info.kind
        );
    }

    // 场景 4: 中间覆盖（分裂原区间）
    println!("\n场景 4: 中间覆盖（分裂原区间）");
    set.clear();
    set.add(10..40, "A")?;
    println!("添加 [10, 40) kind=A");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range.start, info.range.end, info.kind
        );
    }

    set.add(20..30, "B")?;
    println!("添加 [20, 30) kind=B (覆盖中间，分裂 A)");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range.start, info.range.end, info.kind
        );
    }

    // 场景 5: kind 相同时合并
    println!("\n场景 5: kind 相同时合并（对比）");
    set.clear();
    set.add(10..20, "A")?;
    set.add(30..40, "A")?;
    println!("添加 [10, 20) kind=A 和 [30, 40) kind=A");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range.start, info.range.end, info.kind
        );
    }

    set.add(15..35, "A")?;
    println!("添加 [15, 35) kind=A (相同 kind，合并)");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range.start, info.range.end, info.kind
        );
    }

    // 场景 6: 复杂场景 - 多个区间
    println!("\n场景 6: 复杂场景 - 覆盖多个不同 kind 的区间");
    set.clear();
    set.add(0..10, "A")?;
    set.add(10..20, "B")?;
    set.add(20..30, "C")?;
    println!("初始状态:");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range.start, info.range.end, info.kind
        );
    }

    set.add(5..25, "D")?;
    println!("添加 [5, 25) kind=D (覆盖三个区间的部分)");
    for info in set.iter() {
        println!(
            "  [{}, {}) kind={}",
            info.range.start, info.range.end, info.kind
        );
    }

    Ok(())
}
