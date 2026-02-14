fn main() {
    let mut num = 0;
    let res = loop {
        println!("🤣🤣🤣🤣");
        if num == 10 {
            break num * num;
        }
        num += 1;
    };
    println!("res :{res}");
}
