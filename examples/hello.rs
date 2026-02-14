fn main() {
    println!("Hello, World!");
    println!("欢迎来到 Rust 学习之旅！");

    // 使用 lib 中的函数
    let result = rust_learning::add(5, 3);
    println!("5 + 3 = {}", result);

    let greeting = rust_learning::greet("学习者");
    println!("{}", greeting);
}
