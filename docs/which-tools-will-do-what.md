---
title: Which tools will do what?
weight: 4
---

The package will automatically decide which tools to use on a particular image.

## JPGs

JPGs will be made smaller by running them through [Jpegoptim](https://github.com/tjko/jpegoptim). These options are used:

- `-m85`: this will store the image with 85% quality. This setting [seems to satisfy Google's Pagespeed compression rules](https://webmasters.stackexchange.com/questions/102094/google-pagespeed-how-to-satisfy-the-new-image-compression-rules).
- `--force`: writes the result even if it is not smaller. Enable [safe output](usage.md#keeping-the-original-safe) to keep the original in that case.
- `--strip-all`: this strips out all text information such as comments and EXIF data.
- `--all-progressive`: this will make sure the resulting image is a progressive one, meaning it can be downloaded using multiple passes of progressively higher details.

## PNGs

PNGs will be made smaller by running them through two tools. The first one is [Pngquant 2](https://pngquant.org/), a lossy PNG compressor. Unlike the PHP defaults, these options are used:

- `--quality=85`: sets the maximum quality to 85.
- `--force`: allows overwriting the output file.

After that we run the image through a second one: [Optipng](https://optipng.sourceforge.net/). These options are used:

- `-i0`: produces a non-interlaced image.
- `-o2`: this sets the optimization level to two (multiple IDAT compression trials).
- `-quiet`: suppresses normal progress output.

## SVGs

SVGs will be minified by [SVGO](https://github.com/svg/svgo). SVGO's installed default configuration is used; unlike the PHP package, no plugins are disabled. To preserve IDs or a `viewBox`, pass `--config` and the path to your SVGO configuration using `with_options` on `Tool::Svgo`.

Please be aware that SVGO can break your svg. You'll find more info on that in this [excellent blogpost](https://www.sarasoueidan.com/blog/svgo-tools/) by [Sara Soueidan](https://twitter.com/SaraSoueidan). Check the rendered image after changing the configuration, especially when other elements reference its IDs.

## GIFs

GIFs will be optimized by [Gifsicle](https://www.lcdf.org/gifsicle/). These options are used:

- `-b`: writes the optimized image back to the input file.
- `-O3`: this sets the optimization level to Gifsicle's maximum, which produces the slowest but best results.

## WEBPs

WEBPs will be optimized by [cwebp](https://developers.google.com/speed/webp/docs/cwebp). These options are used:

- `-m 6` for the slowest compression method in order to get the best compression.
- `-pass 10` for maximizing the amount of analysis passes.
- `-mt` multithreading for some speed improvements.
- `-q 90` Quality factor that brings the least noticeable changes.

(Settings are originally taken from [here](https://medium.com/@vinhlh/how-i-apply-webp-for-optimizing-images-9b11068db349).)

## AVIFs

AVIFs will be optimized by [avifenc](https://github.com/AOMediaCodec/libavif/blob/main/doc/avifenc.1.md). These options are used:

- `-a cq-level=23`: Constant Quality level. Lower values mean better quality and greater file size (0-63).
- `-j all`: Number of jobs (worker threads, `all` uses all available cores).
- `--min 0`: Min quantizer for color (0-63).
- `--max 63`: Max quantizer for color (0-63).
- `--minalpha 0`: Min quantizer for alpha (0-63).
- `--maxalpha 63`: Max quantizer for alpha (0-63).
- `-a end-usage=q` Rate control mode set to Constant Quality mode.
- `-a tune=ssim`: SSIM as tune the encoder for distortion metric.

(Settings are originally taken from [here](https://web.dev/compress-images-avif/#create-an-avif-image-with-default-settings) and [here](https://github.com/feat-agency/avif).)

Before encoding, `avifdec -j all --ignore-icc --no-strict --png-compress 0` decodes the image into a temporary PNG. Both commands share the optimizer's timeout budget. The PNG is removed on success or error.

AVIF jobs can use many CPU threads by default. Supply `["-j", "1"]` in custom encoder options when resources are limited.

## Changing the quality

`OptimizerChainFactory::with_quality(q)` sets JPEG and PNG quality to `q`, WebP quality to `q`,
and AVIF `cq-level` to `round(63 - q * 0.63)`. Valid quality is 0 through 100.
Without an explicit quality, AVIF uses 23 and WebP uses 90.
