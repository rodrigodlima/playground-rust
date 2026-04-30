fn main() {
    // 1. Immutable borrow: many &T references can coexist
    let s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    println!("immutable: {} {} {}", s, r1, r2);

    // 2. Mutable borrow: needs `mut` on the variable and `&mut` on the reference
    let mut s2 = String::from("hello");
    let r3 = &mut s2;
    r3.push_str(", world");
    println!("mutable: {}", r3);

    // 3. Rule: while a &mut is active, no other borrow is allowed
    //    (uncomment the lines below to see the compiler error)
    // let mut s3 = String::from("hello");
    // let a = &mut s3;
    // let b = &mut s3;        // error: cannot borrow `s3` as mutable more than once
    // println!("{} {}", a, b);

    // 4. After the borrow's last use, the owner can be used again
    let mut s4 = String::from("hello");
    let r4 = &mut s4;
    r4.push_str("!");
    println!("via borrow: {}", r4); // last use of r4
    s4.push_str(" bye");             // ok: r4 is no longer in use
    println!("via owner:  {}", s4);
}
