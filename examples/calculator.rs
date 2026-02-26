use std::io;

enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

fn calc(a: f64, b: f64, op: Op) -> Option<f64> {
    match op {
        Op::Add => Some(a + b),
        Op::Sub => Some(a - b),
        Op::Mul => Some(a * b),
        Op::Div => {
            if b == 0.0 {
                None
            } else {
                Some(a / b)
            }
        }
    }
}

fn str_to_op(s: &str) -> Option<Op> {
    match s {
        "+" => Some(Op::Add),
        "-" => Some(Op::Sub),
        "x" => Some(Op::Mul),
        "/" => Some(Op::Div),
        _ => None,
    }
}

fn main() {
    println!("请输入两个数字");
    let mut num1_str = String::new();
    let mut num2_str = String::new();
    io::stdin().read_line(&mut num1_str).unwrap();
    io::stdin().read_line(&mut num2_str).unwrap();

    let num1 = num1_str.trim().parse().unwrap();
    let num2 = num2_str.trim().parse().unwrap();

    println!("请输入运算符号：(+ - x /)");
    let mut op_str = String::new();
    io::stdin().read_line(&mut op_str).unwrap();
    // 只保留第一位 去除换行符
    if let Some(op) = str_to_op(&op_str[..1]) {
        match calc(num1, num2, op) {
            Some(v) => println!("结果: {v}"),
            None => println!("除数不能为零"),
        }
    }
}
