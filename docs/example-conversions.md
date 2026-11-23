---
title: Example conversions
weight: 6
---

The native-tool integration test optimizes the PHP package's original JPEG, PNG, SVG, GIF, WebP and AVIF
fixtures. It prints before/after sizes, checks each output is smaller and has the expected MIME type,
checks AVIF with the decoder, and verifies every source file remains unchanged.

```sh
cargo test --locked --test native_tools -- --ignored --nocapture
```

Results depend on installed tool versions. The PHP README's published size and DSSIM measurements
are not measurements of this port. See its [original examples](https://github.com/spatie/image-optimizer#example-conversions)
for their source and methodology. The photo fixture is by Jeff Sheldon via Unsplash.
