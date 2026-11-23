---
title: Example conversions
weight: 7
---

Here are some real-life example conversions done by this package.

The JPG, WebP, AVIF, PNG and SVG sources are the same images used by [spatie/image-optimizer](https://github.com/spatie/image-optimizer#example-conversions), optimized with the Rust package's default chain. The photo by [Jeff Sheldon](https://unsplash.com/photos/jTeQavJjBDs) was resized to 2048 pixels wide for the original PHP examples. We used those existing fixtures without resizing them again. The GIF is an additional test fixture in this Rust package.

| Format | Original bytes | Optimized bytes | Reduction |
| --- | ---: | ---: | ---: |
| JPG | 789,989 | 523,422 | 33.74% |
| WebP | 472,102 | 414,036 | 12.30% |
| AVIF | 742,683 | 197,712 | 73.38% |
| PNG | 40,302 | 9,764 | 75.77% |
| SVG | 25,586 | 19,164 | 25.10% |
| GIF | 652,193 | 609,977 | 6.47% |

Sizes are measured in bytes, not rounded kilobytes. These results were measured on Linux on November 23, 2026, with jpegoptim 1.4.7, pngquant 2.18.0, OptiPNG 0.7.8, SVGO 3.3.5, Gifsicle 1.96, cwebp 1.5.0 and libavif 1.2.1 with the AOM 3.12.1 encoder. Results will vary with tool versions and image content. A smaller file does not imply identical pixels; JPEG, PNG, WebP and AVIF optimization can be lossy. No DSSIM score was measured.

## jpg

Original, 789,989 bytes:

![Original JPG photo](/docs/image-optimizer-rust/v1/examples/original-image.jpg)

Optimized, 523,422 bytes (33.74% smaller):

![Optimized JPG photo](/docs/image-optimizer-rust/v1/examples/image.jpg)

## webp

Original, 472,102 bytes:

![Original WebP photo](/docs/image-optimizer-rust/v1/examples/original-image.webp)

Optimized, 414,036 bytes (12.30% smaller):

![Optimized WebP photo](/docs/image-optimizer-rust/v1/examples/image.webp)

## avif

Original, 742,683 bytes:

![Original AVIF photo](/docs/image-optimizer-rust/v1/examples/original-image.avif)

Optimized, 197,712 bytes (73.38% smaller):

![Optimized AVIF photo](/docs/image-optimizer-rust/v1/examples/image.avif)

Photo credit: Jeff Sheldon, via [Unsplash](https://unsplash.com).

## png

Original, Photoshop "Save for web" export with transparency, 40,302 bytes:

![Original PNG logo](/docs/image-optimizer-rust/v1/examples/original-logo.png)

Optimized, 9,764 bytes (75.77% smaller):

![Optimized PNG logo](/docs/image-optimizer-rust/v1/examples/logo.png)

## svg

Original, Illustrator web-optimized export, 25,586 bytes:

![Original SVG graph](/docs/image-optimizer-rust/v1/examples/original-graph.svg)

Optimized, 19,164 bytes (25.10% smaller):

![Optimized SVG graph](/docs/image-optimizer-rust/v1/examples/graph.svg)

## gif

Original, 652,193 bytes:

![Original animated GIF](/docs/image-optimizer-rust/v1/examples/original-animated.gif)

Optimized, 609,977 bytes (6.47% smaller):

![Optimized animated GIF](/docs/image-optimizer-rust/v1/examples/animated.gif)

## Trying it yourself

Install the [optimization tools](installation-and-setup.md#optimization-tools), clone the repository and run this code from its root. It writes the results to `optimized-examples` and leaves the source images untouched:

```rust,no_run
use spatie_image_optimizer::OptimizerChainFactory;
use std::{fs, path::Path};

fs::create_dir_all("optimized-examples")?;
for name in ["image.jpg", "image.webp", "image.avif", "logo.png", "graph.svg", "animated.gif"] {
    let source = Path::new("tests/fixtures").join(name);
    let output = Path::new("optimized-examples").join(name);
    let report = OptimizerChainFactory::create()
        .safe_output(true)
        .throws(true)
        .optimize_to_with_report(&source, &output)?;
    println!("{name}: {} -> {} bytes", report.original_bytes, fs::metadata(output)?.len());
}
# Ok::<(), spatie_image_optimizer::Error>(())
```
