---
title: Installation
weight: 3
---

You can install the package from GitHub. Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
spatie-image-optimizer = { git = "https://github.com/spatie/image-optimizer-rust", branch = "main" }
```

Requires Rust 1.89 or newer. The crate is not yet published to crates.io. Commit your application's `Cargo.lock` to keep the selected Git revision between builds.

<a id="optimization-tools"></a>

## Optimization tools

The package uses these optimizers for matching image formats:

- [Jpegoptim](https://github.com/tjko/jpegoptim)
- [Optipng](https://optipng.sourceforge.net/)
- [Pngquant](https://pngquant.org/)
- [SVGO](https://github.com/svg/svgo)
- [Gifsicle](https://www.lcdf.org/gifsicle/)
- [cwebp](https://developers.google.com/speed/webp/docs/precompiled)
- [avifdec and avifenc](https://github.com/AOMediaCodec/libavif)

Here's how to install all the optimizers on Ubuntu or Debian:

```sh
sudo apt-get install jpegoptim optipng pngquant gifsicle webp libavif-bin
npm install -g svgo@3
```

And here's how to install the binaries on macOS using [Homebrew](https://brew.sh/):

```sh
brew install jpegoptim optipng pngquant gifsicle webp libavif
npm install -g svgo@3
```

On Fedora:

```sh
sudo dnf install jpegoptim optipng pngquant gifsicle libwebp-tools libavif-tools
npm install -g svgo@3
```

Install only the formats you need. AVIF needs both `avifdec` and `avifenc` (libavif 0.9.3 or newer
with the documented options). On Windows install native executables and add them to `PATH`, or use
`set_binary_path`. Node is required only for the external SVGO tool, not for the Rust library.
