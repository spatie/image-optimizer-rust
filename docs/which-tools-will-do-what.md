---
title: Which tools will do what?
weight: 4
---

The package will automatically decide which tools to use on a particular image.

## JPGs

JPGs will be made smaller by running them through [Jpegoptim](https://github.com/tjko/jpegoptim). These options are used:

- `-m85`: limits the image quality to 85.
- `--force`: writes the result even if it is not smaller. Enable [safe output](usage.md#keeping-the-original-safe) to keep the original in that case.
- `--strip-all`: strips text information such as comments and EXIF data.
- `--all-progressive`: produces a progressive JPEG, which can be downloaded in multiple passes of progressively higher detail.

## PNGs

PNGs will be made smaller by running them through two tools. The first is [Pngquant](https://pngquant.org/), a lossy PNG compressor. These options are used:

- `--quality=85`: sets the maximum quality to 85.
- `--force`: allows overwriting the output file.

After that we run the image through [Optipng](https://optipng.sourceforge.net/), a lossless optimizer. These options are used:

- `-i0`: produces a non-interlaced image.
- `-o2`: sets the optimization level to two, trying multiple compression strategies.
- `-quiet`: suppresses normal progress output.

## SVGs

SVGs will be minified by [SVGO](https://github.com/svg/svgo). SVGO's installed default configuration is used; no plugins are disabled by the package. To preserve IDs or a `viewBox`, pass your own configuration:

```rust,no_run
use spatie_image_optimizer::{OptimizerChain, Tool};

OptimizerChain::new()
    .add_optimizer(Tool::Svgo.with_options(["--config", "/path/to/svgo.config.mjs"]))
    .optimize("graph.svg")?;
# Ok::<(), spatie_image_optimizer::Error>(())
```

Please be aware that SVGO can break your SVG, especially when other elements reference its IDs. You'll find more information in [this blog post by Sara Soueidan](https://www.sarasoueidan.com/blog/svgo-tools/). Check the rendered image after changing the configuration.

## GIFs

GIFs will be optimized by [Gifsicle](https://www.lcdf.org/gifsicle/). These options are used:

- `-b`: writes the optimized image back to the input file.
- `-O3`: sets the optimization level to Gifsicle's maximum, which produces the slowest but best compression.

## WEBPs

WEBPs will be optimized by [cwebp](https://developers.google.com/speed/webp/docs/cwebp). These options are used:

- `-m 6`: uses the slowest compression method for the best compression.
- `-pass 10`: runs ten analysis passes.
- `-mt`: uses multithreading to speed up compression.
- `-q 90`: sets the quality factor to 90.

## AVIFs

AVIFs will be optimized by [avifenc](https://github.com/AOMediaCodec/libavif/blob/main/doc/avifenc.1.md). These options are used:

- `-a cq-level=23`: sets the constant quality level. Lower values mean better quality and larger files (0-63).
- `-j all`: uses all available CPU cores.
- `--min 0` and `--max 63`: set the minimum and maximum quantizer for color.
- `--minalpha 0` and `--maxalpha 63`: set the minimum and maximum quantizer for alpha.
- `-a end-usage=q`: uses constant quality rate control.
- `-a tune=ssim`: tunes the encoder for the SSIM distortion metric.

Before encoding, `avifdec -j all --ignore-icc --no-strict --png-compress 0` decodes the image into a temporary PNG. Both commands share the optimizer's timeout budget. The PNG is removed on success or error.

AVIF jobs can use many CPU threads by default. Supply `["-j", "1"]` in custom encoder options when resources are limited.

## Changing the quality

`OptimizerChainFactory::with_quality(q)` sets JPEG and PNG quality to `q`, WebP quality to `q`,
and AVIF `cq-level` to `round(63 - q * 0.63)`. Valid quality is 0 through 100.
Without an explicit quality, AVIF uses 23 and WebP uses 90.
