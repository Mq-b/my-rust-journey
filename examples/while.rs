fn test_while() -> i32 {
    let mut n: i32 = 0;
    while n < 10 {
        println!("🤣👉🐢");
        n += 1;
    }
    return n;
}

fn main() {
    let res = test_while();
    println!("res: {res}");
}
