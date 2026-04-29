# playground-rust

POC about Rust language.

This repository is organized as a [Cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html). Each subdirectory is a standalone crate used to experiment with a specific Rust topic.

## Adding a new project to the workspace

Follow these steps whenever you want to add a new crate (replace `<crate-name>` with the name of the new project, e.g. `debug-formater`).

### 1. Create the crate

From the repository root:

```bash
cargo init --name <crate-name> <crate-name>
```

- `--name <crate-name>` sets the package name explicitly (otherwise Cargo would derive it from the directory name).
- The trailing `<crate-name>` is the path where the crate is created.
- Omit `--lib` to generate a binary crate (default), matching the convention of the other workspace members. Use `--lib` if you want a library instead.

### 2. Register the crate in the workspace

`cargo init` is workspace-aware and automatically appends the new crate to the `members` list in the root `Cargo.toml`:

```toml
[workspace]
members = [
    "<crate-name>",
    "hello-world",
    "segment-tree",
    "struct-sum-example",
    "variables-and-data-types",
]
resolver = "2"
```

If for any reason the entry is missing, add it manually.

### 3. Align the manifest with the repo conventions

The other crates in this workspace pin `version = "0.0.1"` and `edition = "2024"`. Update `<crate-name>/Cargo.toml` to match:

```toml
[package]
name = "<crate-name>"
version = "0.0.1"
edition = "2024"

[dependencies]
```

### 4. Build and run

From the repository root, target the package with the `-p` flag:

```bash
cargo build -p <crate-name>
cargo run   -p <crate-name>
```

The default scaffold prints `Hello, world!`, confirming the crate is wired up correctly.
