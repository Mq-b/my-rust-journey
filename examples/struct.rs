use std::fmt;
struct Rectangle {
    width: f64,
    height: f64,
}

impl Rectangle {
    fn new(width: f64, height: f64) -> Self {
        return Rectangle { width, height };
    }
    fn area(&self) -> f64 {
        return self.width * self.height;
    }
    fn perimeter(&self) -> f64 {
        return (self.width + self.height) * 2.0;
    }
    fn is_square(&self) -> bool {
        return self.width == self.height;
    }
}

impl fmt::Display for Rectangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rectangle({} x {})", self.width, self.height)
    }
}

fn main() {
    let rec = Rectangle::new(10.0, 20.0);

    println!("area:{}", rec.area());
    println!("perimeter:{}", rec.perimeter());
    println!("is_square:{}", rec.is_square());
    println!("Rec: {}", rec);
}
