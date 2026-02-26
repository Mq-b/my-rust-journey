enum E {
    A,
    B,
    C,
}

fn test_value(i: E) {
    match i {
        E::A => println!("1"),
        E::B => println!("2"),
        E::C => println!("3"),
    }
}

fn test_option(o: Option<i32>) -> Option<i32> {
    match o {
        None => {
            println!("o为空");
            return None;
        }
        Some(3) => {
            println!("o为3");
            return Some(3);
        }
        Some(i) => {
            println!("o有值，返回o+1");
            return Some(i + 1);
        }
    }
}

fn main() {
    test_value(E::B);

    if let Some(1) = test_option(Some(1)) {
        println!("匹配");
    } else {
        println!("不匹配");
    }

    if let Some(a) = test_option(Some(3)) {
        println!("匹配 {a}");
    } else {
        println!("不匹配");
    }

    test_option(None);
}
