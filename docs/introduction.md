---
title: Introduction
weight: 1
---

This package can optimize PNGs, JPGs, WEBPs, AVIFs, SVGs and GIFs by running them through a chain of various [image optimization tools](which-tools-will-do-what.md). Here's how you can use it:

```rust,no_run
use spatie_image_optimizer::OptimizerChainFactory;

let optimizer_chain = OptimizerChainFactory::create();
optimizer_chain.optimize("photo.jpg")?;
# Ok::<(), spatie_image_optimizer::Error>(())
```

The image at `photo.jpg` will be overwritten by an optimized version. The package automatically decides which tools to use for the image's format. Install the [binaries you need](installation-and-setup.md#optimization-tools) first. By default, failures, including missing binaries, are logged and the chain continues.

To keep the original image, use `optimize_to("photo.jpg", "optimized.jpg")`. If you only want to replace a file when optimization succeeds and makes it smaller, enable [safe output](usage.md#keeping-the-original-safe).

Here are some [example conversions](example-conversions.md) that have been done by this package.
