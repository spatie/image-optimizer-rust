---
title: Introduction
weight: 1
---

This package can optimize PNGs, JPGs, WEBPs, AVIFs, SVGs and GIFs by running them through a chain of various [image optimization tools](which-tools-will-do-what.md). Here's how you can use it:

```rust,no_run
# fn main() -> Result<(), spatie_image_optimizer::Error> {
# let directory = tempfile::tempdir()?;
# let source = directory.path().join("image.jpg");
# std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/image.jpg"), &source)?;
# let path_to_image = source.as_path();
use spatie_image_optimizer::OptimizerChainFactory;

let optimizer_chain = OptimizerChainFactory::create();

optimizer_chain.optimize(path_to_image)?;
# assert_eq!(spatie_image_optimizer::Image::new(&source)?.mime()?, "image/jpeg");
# Ok(())
# }
```

The image at `path_to_image` will be overwritten by an optimized version which should be smaller. The package will automatically decide which tools to use on a particular image.

Here are some [example conversions](example-conversions.md) that have been done by this package.

Install the [optimization tools](installation-and-setup.md#optimization-tools) first. See [usage](usage.md) to keep the original image or enable safe output.
