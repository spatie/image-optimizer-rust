---
title: Which tools will do what?
weight: 3
---

## JPGs

`jpegoptim -m85 --force --strip-all --all-progressive` reduces quality to at most 85,
removes metadata and produces progressive JPEGs.

## PNGs

`pngquant --quality=85 --force` runs first, then `optipng -i0 -o2 -quiet`.
Pngquant is lossy; Optipng performs lossless optimization. Both write to the original path.

## SVGs

SVGO runs with no extra options and uses its installed default configuration.
This follows the PHP factory implementation, which does not actually disable any plugins.
To preserve IDs or a viewBox, pass your own SVGO configuration using `Tool::Svgo.with_options(...)`.
Review the rendered SVG if its IDs are referenced from elsewhere.

## GIFs

`gifsicle -b -O3` uses maximum optimization and writes the output back to the image.
No interlace option is added.

## WEBPs

`cwebp -m 6 -pass 10 -mt -q 90` uses the slowest compression method, ten analysis passes,
multithreading and quality 90.

## AVIFs

The decoder first runs `avifdec -j all --ignore-icc --no-strict --png-compress 0` into a
reserved temporary PNG. The encoder then runs:

```text
avifenc -a cq-level=23 -j all --min 0 --max 63 --minalpha 0 --maxalpha 63 -a end-usage=q -a tune=ssim
```

Both commands share the optimizer's timeout budget. The private PNG is removed on success or error.
AVIF jobs can use many CPU threads by default. Supply `-j 1` in custom options when resources are limited.

`OptimizerChainFactory::with_quality(q)` sets JPEG and PNG quality to `q`, WebP quality to `q`,
and AVIF `cq-level` to `round(63 - q * 0.63)`. Valid quality is 0 through 100.
Without an explicit quality, AVIF uses 23 and WebP uses 90, as in PHP.
