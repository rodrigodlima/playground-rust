fn main() {
    let v = vec![1, 2, 3];
    println!("Debug: {:?}", v);
    //println!("Normal: {}", v); Here, it will gererate a compilation error, because Vec<T> and HashMap only implement Debug
}

use std::fmt;

struct Ponto { x: i32, y: i32 }

impl fmt::Display for Ponto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
