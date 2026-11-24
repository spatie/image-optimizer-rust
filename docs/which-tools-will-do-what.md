---
title: Which tools will do what?
weight: 4
---

The package will automatically decide which tools to use on a particular image.

## JPGs

JPGs will be made smaller by running them through [Jpegoptim](https://github.com/tjko/jpegoptim). These options are used:

- `-m85`: this will store the image with 85% quality. This setting [seems to satisfy Google's Pagespeed compression rules](https://webmasters.stackexchange.com/questions/102094/google-pagespeed-how-to-satisfy-the-new-image-compression-rules).
- `--force`: writes the result even if it is not smaller. See [safe output](usage.md#keeping-the-original-safe).
- `--strip-all`: this strips out all text information such as comments and EXIF data.
- `--all-progressive`: this will make sure the resulting image is a progressive one, meaning it can be downloaded using multiple passes of progressively higher details.

## PNGs

PNGs will be made smaller by running them through two tools. The first one is [Pngquant 2](https://pngquant.org/), a lossy PNG compressor. These options are used:

- `--quality=85`: sets the maximum quality to 85.
- `--force`: allows overwriting the output file.

After that we run the image through a second one: [Optipng](https://optipng.sourceforge.net/). These options are used:

- `-i0`: produces a non-interlaced image.
- `-o2`: this sets the optimization level to two (multiple IDAT compression trials).
- `-quiet`: suppresses normal progress output.

## SVGs

SVGs will be minified by [SVGO](https://github.com/svg/svgo) with its installed default configuration. To preserve IDs or a `viewBox`, pass `--config` and the path to your configuration using `Tool::Svgo.with_options`. Check the rendered image, especially when other elements reference its IDs.

Please be aware that SVGO can break your svg. You'll find more info on that in this [excellent blogpost](https://www.sarasoueidan.com/blog/svgo-tools/) by [Sara Soueidan](https://twitter.com/SaraSoueidan).

## GIFs

GIFs will be optimized by [Gifsicle](https://www.lcdf.org/gifsicle/). These options will be used:

- `-b`: writes the optimized image back to the input file.
- `-O3`: this sets the optimization level to Gifsicle's maximum, which produces the slowest but best results.

## WEBPs

WEBPs will be optimized by [Cwebp](https://developers.google.com/speed/webp/docs/cwebp). These options will be used:

- `-m 6` for the slowest compression method in order to get the best compression.
- `-pass 10` for maximizing the amount of analysis passes.
- `-mt` multithreading for some speed improvements.
- `-q 90` Quality factor that brings the least noticeable changes.

(Settings are originally taken from [here](https://medium.com/@vinhlh/how-i-apply-webp-for-optimizing-images-9b11068db349).)

## AVIFs

AVIFs will be optimized by [avifenc](https://github.com/AOMediaCodec/libavif/blob/main/doc/avifenc.1.md). These options will be used:

- `-a cq-level=23`: Constant Quality level. Lower values mean better quality and greater file size (0-63).
- `-j all`: Number of jobs (worker threads, `all` uses all available cores).
- `--min 0`: Min quantizer for color (0-63).
- `--max 63`: Max quantizer for color (0-63).
- `--minalpha 0`: Min quantizer for alpha (0-63).
- `--maxalpha 63`: Max quantizer for alpha (0-63).
- `-a end-usage=q` Rate control mode set to Constant Quality mode.
- `-a tune=ssim`: SSIM as tune the encoder for distortion metric.

(Settings are originally taken from [here](https://web.dev/compress-images-avif/#create-an-avif-image-with-default-settings) and [here](https://github.com/feat-agency/avif).)

AVIF also needs `avifdec`. It decodes into a temporary PNG using `-j all --ignore-icc --no-strict --png-compress 0`. Both processes share the optimizer's timeout budget and the PNG is removed on success or error. Set encoder options to `["-j", "1"]` when CPU resources are limited.

## Changing the quality

Use `OptimizerChainFactory::with_quality(q)?` with a quality from 0 through 100. It sets JPEG, PNG and WebP quality to `q`, and AVIF `cq-level` to `round(63 - q * 0.63)`.
