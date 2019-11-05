# srcfiles
A tool for searching source files used to compile a Rust crate.

# Usage
srcfiles path/to/root.rs

Prints best-effort representation of all .rs and `include!` files used to compile a crate (currently only with debug output)