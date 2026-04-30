# Borrowing

A small playground covering Rust's **ownership** and **borrowing** rules.

## Why borrowing exists

Rust has no garbage collector. Memory safety is enforced at compile time through three rules:

1. Every value has a single **owner**.
2. When the owner goes out of scope, the value is dropped.
3. Other code can **borrow** the value via references (`&T` or `&mut T`) without taking ownership.

This is what prevents double-frees, use-after-free, and data races — without a runtime cost.

## Move vs. borrow

```rust
let s = String::from("hello");
let r = s;          // MOVE: r is the new owner, s is invalidated
println!("{}", s);  // compile error: borrow of moved value `s`
```

```rust
let s = String::from("hello");
let r = &s;             // BORROW: r is a reference, s is still the owner
println!("{} {}", s, r); // both work
```

`String` owns a heap allocation, so Rust forbids two owners (it would lead to a double-free on drop). Assignment moves ownership instead of copying.

### Why some types don't move

```rust
let n = 5;
let m = n;
println!("{} {}", n, m); // works
```

Types with a fixed size on the stack (`i32`, `bool`, `char`, tuples of `Copy` types, etc.) implement the `Copy` trait. For them, `=` copies the bits cheaply rather than moving. `String`, `Vec<T>`, `Box<T>`, and other heap-owning types are **not** `Copy`.

## The two kinds of references

| Kind         | Syntax    | How many at once | Can mutate? |
|--------------|-----------|------------------|-------------|
| Immutable    | `&T`      | Many             | No          |
| Mutable      | `&mut T`  | Exactly one      | Yes         |

And the cross-rule: **you cannot have an immutable and a mutable borrow active at the same time.**

Mental model: *shared read OR exclusive write — never both*. This is the same invariant that prevents data races in concurrent code, applied uniformly.

### Immutable borrow

```rust
let s = String::from("hello");
let r1 = &s;
let r2 = &s;        // fine, as many as you want
println!("{} {} {}", s, r1, r2);
```

### Mutable borrow

```rust
let mut s = String::from("hello");
let r = &mut s;
r.push_str(", world");
```

Two requirements: the binding must be `mut` (`let mut s`) **and** the reference must be `&mut`. One without the other won't compile.

### What the compiler rejects

```rust
let mut s = String::from("hello");
let a = &mut s;
let b = &mut s;          // error: second mutable borrow
println!("{} {}", a, b);
```

```rust
let mut s = String::from("hello");
let a = &s;
let b = &mut s;          // error: mutable borrow while immutable borrow is alive
println!("{} {}", a, b);
```

## Non-Lexical Lifetimes (NLL)

A borrow ends at its **last use**, not at the end of the scope. This makes a lot of "obviously fine" code actually compile:

```rust
let mut s = String::from("hello");
let r = &mut s;
r.push_str("!");
println!("{}", r);   // last use of r — borrow ends here
s.push_str(" bye");  // ok: s is no longer borrowed
```

Before NLL (Rust ≤ 2018 edition's old borrow checker), this was rejected because the borrow lived until the end of the block.

## Running the example

```bash
cargo run -p borrowing
```

The four numbered sections in `src/main.rs` correspond directly to the cases above.
