---
title: Example conversions
weight: 7
---

Here are some real life example conversions done by this package.

Methodology for JPG, WEBP, AVIF images: the [original image](https://unsplash.com/photos/jTeQavJjBDs) has been fed to [spatie/image](https://github.com/spatie/image) (using the default GD driver) and resized to 2048px width.

These are the existing [PHP fixtures](https://github.com/spatie/image-optimizer#example-conversions), optimized with the Rust package's default chain without resizing again. The GIF is an additional Rust test fixture.

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

Original: Photoshop 'Save for web' | PNG-24 with transparency<br>
40,302 bytes

![Original PNG logo](/docs/image-optimizer-rust/v1/examples/original-logo.png)

Optimized, 9,764 bytes (75.77% smaller):

![Optimized PNG logo](/docs/image-optimizer-rust/v1/examples/logo.png)

## svg

Original: Illustrator | Web optimized SVG export<br>
25,586 bytes

![Original SVG graph](/docs/image-optimizer-rust/v1/examples/original-graph.svg)

Optimized, 19,164 bytes (25.10% smaller):

![Optimized SVG graph](/docs/image-optimizer-rust/v1/examples/graph.svg)

## gif

Original, 652,193 bytes:

![Original animated GIF](/docs/image-optimizer-rust/v1/examples/original-animated.gif)

Optimized, 609,977 bytes (6.47% smaller):

![Optimized animated GIF](/docs/image-optimizer-rust/v1/examples/animated.gif)

To try these conversions yourself, use the [default chain](usage.md) with the images in `tests/fixtures`. Use `optimize_to` to keep the source images untouched. The [native tests](testing.md) run every format and check the output.
