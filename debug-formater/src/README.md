# {:?} — Debug formatter

This is the Debug format specifier used in macros such as `println!`, `format!`, `eprintln!`, `dbg!`, etc. It prints the value using the `Debug` trait implementation (not `Display`, which is the "regular" `{}`).

Practical difference: `Display` (`{}`) is the "end-user friendly" representation; `Debug` (`{:?}`) is the "developer / log" representation. That is why `String` implements `Display` (prints the text) and also `Debug` (prints it inside quotes, showing escape sequences).
