use ranges_ext::RangeSet;

fn main() {
    let mut set = RangeSet::new();
    set.add(1..5, "first");
    set.add(3..8, "second");
    set.add(10..15, "third");
    set.add(12..18, "fourth");

    println!("=== Display Output (只显示合并范围) ===");
    for (i, element) in set.elements().iter().enumerate() {
        println!("Element {}: {}", i, element);
    }

    println!("\n=== 十六进制格式 ({{:#x}}) ===");
    for (i, element) in set.elements().iter().enumerate() {
        println!("Element {}: {:#x}", i, element);
    }

    println!("\n=== 二进制格式 ({{:#b}}) ===");
    for (i, element) in set.elements().iter().enumerate() {
        println!("Element {}: {:#b}", i, element);
    }

    println!("\n=== 八进制格式 ({{:#o}}) ===");
    for (i, element) in set.elements().iter().enumerate() {
        println!("Element {}: {:#o}", i, element);
    }

    println!("\n=== Debug Output (详细信息) ===\n");
    for (i, element) in set.elements().iter().enumerate() {
        println!("Element {}:", i);
        println!("{:#?}\n", element);
    }
}
