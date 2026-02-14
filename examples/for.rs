use std::io;

fn main() {
    let mut num_str = String::new();
    io::stdin().read_line(&mut num_str).unwrap();
    let num: i32 = num_str.trim().parse().unwrap();
    for item in 0..num {
        print!("{item} ");
    }
    println!();

    let mut arr = [1, 2, 3, 4];
    for value in arr {
        print!("{value} ");
    }
    println!();

    for (index, value) in arr.iter().enumerate() {
        println!("arr[{index}]={value}");
    }
    println!();

    // 遍历大元素需要按引用，避免拷贝增加性能
    for value in &arr {
        print!("{value} ");
    }
    println!();

    for value in &mut arr {
        *value += 1;
    }
    println!("array:{:?}", arr);
}
