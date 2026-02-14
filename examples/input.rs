use std::io;
#[allow(dead_code)]
fn test_input_string() {
    let mut guess = String::new();
    io::stdin()
        .read_line(&mut guess)
        .expect("Failed to read line");
    println!("User Input: {guess}");
}

fn test_input_int_array() {
    let mut v: Vec<i32> = Vec::new();
    let mut length_str = String::new();
    println!("Please Input Array length");

    io::stdin()
        .read_line(&mut length_str)
        .expect("Failed to read line");
    let length: i32 = length_str.trim().parse().expect("不是数字串");
    println!("Please Input Array");
    for _ in 0..length {
        let mut str = String::new();
        io::stdin().read_line(&mut str).expect("Failed to read");
        v.push(str.trim().parse().expect("不是数字串"));
    }
    println!("Array Length: {length} value: {v:?}")
}

fn main() {
    println!("Hello, world!");
    //test_input_string();
    test_input_int_array();
}
