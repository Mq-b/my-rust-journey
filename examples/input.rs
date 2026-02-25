use std::io;

#[allow(dead_code)]
fn test_input_string() {
    let mut guess = String::new();
    io::stdin()
        .read_line(&mut guess)
        .expect("Failed to read line");
    println!("User Input: {guess}");
}

fn test_input_int_array() -> anyhow::Result<()> {
    let mut v: Vec<i32> = Vec::new();
    let mut length_str = String::new();
    println!("Please Input Array length");

    io::stdin()
        .read_line(&mut length_str)
        .expect("Failed to read line");

    let mut length: i32 = 0;
    length = match length_str.trim().parse() {
        Ok(num) => num,
        Err(_) => return Err(anyhow::anyhow!("不是数字串!")),
    };

    println!("Please Input Array");
    for _ in 0..length {
        let mut str = String::new();
        io::stdin().read_line(&mut str).expect("Failed to read");
        v.push(str.trim().parse().expect("不是数字串"));
    }
    println!("Array Length: {length} value: {v:?}");
    Ok(())
}

fn main() {
    println!("Hello, world!");
    //test_input_string();
    match test_input_int_array() {
        Ok(()) => println!("成功!"),
        Err(e) => println!("失败: {e}"),
    }
}
