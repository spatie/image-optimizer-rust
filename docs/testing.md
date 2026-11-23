---
title: Testing
weight: 9
---

Run `cargo test --locked --all-targets` and `cargo test --locked --doc` without external tools.
The native suite requires all tools and runs separately with
`cargo test --locked --test native_tools -- --ignored --nocapture`.

CI checks Linux, macOS and Windows on stable, Rust 1.89 on Linux, rustfmt, strict Clippy and
warning-free rustdoc. A dedicated Linux job installs codecs and SVGO and runs the native suite.
