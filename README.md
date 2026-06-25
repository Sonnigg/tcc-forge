# tcc-forge

A lightweight, ergonomic C code generation library for Rust designed for seamless, zero-copy compilation with the Tiny C Compiler (TCC).

[![Crate](https://img.shields.io/badge/crates.io-v0.1.0-blue.svg)](https://crates.io/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tcc-forge` is an ultra-lightweight, high-performance, and ergonomic C code generation utility built for Rust developers who need to dynamically emit standard-compliant C source code. It is specifically optimized to pair with the **Tiny C Compiler (TCC)** toolchain, offering native streaming compilation via standard input pipelines to fully bypass the overhead of writing intermediate source files to disk.

---

## ⚡ Key Features

* **Zero Disk I/O Compilation:** Directly pipes strings into TCC using memory streams (`-x c -`), optimizing compilation speeds for JIT-like workflows, dynamic scripting runtimes, and hot-swapping binaries.
* **Rust-Ergonomic DSL:** Leverages a safe Builder pattern wrapper for C definitions, types, and function structures, shielding you from trivial syntax bugs while maintaining clean C formatting.
* **Zero Dependencies:** Built exclusively on top of the Rust standard library (`std`). It will not bloat your crate's compile times or dependency graph.
* **Flexible Raw Escapes:** Allows blending structured structural builders with inline raw strings (`.raw()`) to effortlessly write complex loops, logic branches, or custom optimization attributes.

---

## 🚀 Installation

Add the following lines directly to your `Cargo.toml` file:

```toml
[dependencies]
tcc-forge = { git = "[https://github.com/yourusername/tcc-forge.git](https://github.com/yourusername/tcc-forge.git)" }