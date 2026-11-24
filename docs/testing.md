---
title: Testing
weight: 9
---

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo test --locked --doc --all-features
cargo doc --locked --no-deps --all-features
cargo package --locked
```

To run the native conversions and documentation examples, install the [optimization tools](installation-and-setup.md#optimization-tools):

```sh
cargo test --locked --test native_tools -- --ignored --nocapture
cargo test --locked --test documentation -- --ignored
```

The [README testing section](https://github.com/spatie/image-optimizer-rust#testing) describes example setup, native test coverage and CI checks.

Set `RUSTFLAGS` and `RUSTDOCFLAGS` to `-D warnings` to match CI. Check the minimum supported Rust version with `cargo +1.89.0 test --locked --all-targets --all-features` and `cargo +1.89.0 test --locked --doc --all-features`.

Install `cargo-audit` with `cargo install cargo-audit --locked`, then run `cargo audit`. CI checks security advisories on Linux.
