---
title: Testing
weight: 9
---

You can run the tests with Cargo:

```sh
cargo test --locked --all-targets
cargo test --locked --doc
```

These tests don't require external tools. To test real image conversions, install all the [optimization tools](installation-and-setup.md#optimization-tools) and run:

```sh
cargo test --locked --test native_tools -- --ignored --nocapture
```

The native tests check that all six formats shrink, source files remain unchanged, and AVIF output can be decoded. They also check quality settings and cleanup after an encoder failure.

CI runs the tests on Linux, macOS and Windows, checks Rust 1.89 on Linux, and runs rustfmt, Clippy and rustdoc. A separate Linux job runs the native tests with the tools installed.
