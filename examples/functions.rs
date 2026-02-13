//! 函数示例
//!
//! 运行方式: cargo run --example functions

fn main() {
    println!("=== 函数示例 ===\n");

    // 1. 无参数函数
    greet();

    // 2. 带参数的函数
    let result = add(5, 3);
    println!("5 + 3 = {}", result);

    // 3. 多个参数
    print_info("Alice", 25);

    // 4. 表达式作为返回值
    let x = {
        let y = 3;
        y + 1 // 注意：没有分号，这是表达式
    };
    println!("表达式块的值: {}", x);

    // 5. 带返回值的函数
    let square = square(4);
    println!("4 的平方: {}", square);

    // 6. 多个返回值（使用元组）
    let (sum, product) = calculate(10, 5);
    println!("10 和 5 的和: {}, 积: {}", sum, product);
}

fn greet() {
    println!("Hello from a function!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b // 返回表达式，无分号
}

fn print_info(name: &str, age: u32) {
    println!("姓名: {}, 年龄: {}", name, age);
}

fn square(x: i32) -> i32 {
    x * x
}

fn calculate(a: i32, b: i32) -> (i32, i32) {
    (a + b, a * b)
}
