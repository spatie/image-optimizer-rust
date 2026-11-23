---
title: Introduction
weight: 1
---

This package can optimize PNGs, JPGs, WEBPs, AVIFs, SVGs and GIFs by running them through a chain of various [image optimization tools](which-tools-will-do-what.md). Here's how you can use it:

```rust,ignore
use spatie_image_optimizer::OptimizerChainFactory;

let optimizer_chain = OptimizerChainFactory::create();

optimizer_chain.optimize(path_to_image)?;
```

The image at `path_to_image` will be overwritten by an optimized version which should be smaller. The package will automatically decide which tools to use on a particular image. Install the [binaries you need](installation-and-setup.md#optimization-tools) first. Missing binaries are logged and the chain continues; they are not skipped by an installation check.

Here are some [example conversions](example-conversions.md) that have been done by this package.

To keep the original image, use `optimize_to(path_to_image, path_to_output)`. If you only want to replace a file when optimization succeeds and makes it smaller, enable [safe output](usage.md#keeping-the-original-safe).
