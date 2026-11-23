---
title: Installation and setup
weight: 2
---

Requires Rust 1.89 or newer. This crate is not yet published to crates.io.

```toml
[dependencies]
spatie-image-optimizer = { path = "../rust-image-optimizer" }
```

## Optimization tools

On Ubuntu or Debian:

```sh
sudo apt-get install jpegoptim optipng pngquant gifsicle webp libavif-bin
npm install -g svgo@3
```

On macOS with Homebrew:

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
