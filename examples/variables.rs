//! 变量和数据类型示例
//!
//! 运行方式: cargo run --example variables

fn main() {
    println!("=== 变量和数据类型 ===\n");

    // 1. 不可变变量（默认）
    let x = 5;
    println!("不可变变量 x = {}", x);

    // 2. 可变变量
    let mut y = 10;
    println!("可变变量 y = {}", y);
    y = 20;
    println!("修改后 y = {}", y);

    // 3. 常量
    const MAX_POINTS: u32 = 100_000;
    println!("常量 MAX_POINTS = {}", MAX_POINTS);

    // 4. 数据类型
    let integer: i32 = 42;
    let float: f64 = 3.14;
    let boolean: bool = true;
    let character: char = '😀';

    println!("\n整数: {}", integer);
    println!("浮点数: {}", float);
    println!("布尔值: {}", boolean);
    println!("字符: {}", character);

    // 5. 元组
    let tuple: (i32, f64, char) = (500, 6.4, 'A');
    let (a, b, c) = tuple; // 解构
    println!("\n元组解构: a={}, b={}, c={}", a, b, c);
    println!("元组访问: tuple.0={}", tuple.0);

    // 6. 数组
    let array = [1, 2, 3, 4, 5];
    println!("\n数组第一个元素: {}", array[0]);
    println!("数组长度: {}", array.len());

    // 7. 字符串
    let s1 = "hello"; // &str 字符串切片
    let s2 = String::from("world"); // String 类型
    println!("\n字符串切片: {}", s1);
    println!("String 类型: {}", s2);
}
