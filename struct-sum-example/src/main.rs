pub struct Sum {
    value: i8,
}

fn main() {
    let grade1 = Sum {
        value: 10,
    };
    let grade2 = Sum {
        value: 9,
    };
    let total = grade1 + grade2;
     println!("{}", total); // ou println!("{total}");
}
