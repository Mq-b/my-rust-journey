//! Rust Learning Library
//!
//! 这是一个用于学习 Rust 的项目库

/// 一个简单的加法函数示例
///
/// # Examples
///
/// ```
/// use rust_learning::add;
/// assert_eq!(add(2, 3), 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// 一个简单的问候函数
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
    }

    #[test]
    fn test_greet() {
        assert_eq!(greet("Rust"), "Hello, Rust!");
    }
}
