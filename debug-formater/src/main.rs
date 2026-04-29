fn main() {
    let v = vec![1, 2, 3];
    println!("Debug: {:?}", v);
    //println!("Normal: {}", v); Here, it will gererate a compilation error, because Vec<T> and HashMap only implement Debug
}
