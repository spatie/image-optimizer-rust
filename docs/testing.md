---
title: Testing
weight: 9
---

```sh
cargo test --locked --all-targets
cargo test --locked --doc
```

To run the native conversions and documentation examples, install the [optimization tools](installation-and-setup.md#optimization-tools):

```sh
cargo test --locked --test native_tools -- --ignored --nocapture
cargo test --locked --test documentation -- --ignored
```

The [README testing section](https://github.com/spatie/image-optimizer-rust#testing) describes example setup, native test coverage and CI checks.
