# Easily optimize images using Rust

[![Latest version on crates.io](https://img.shields.io/crates/v/spatie-image-optimizer.svg?style=flat-square)](https://crates.io/crates/spatie-image-optimizer)
[![Tests](https://img.shields.io/github/actions/workflow/status/spatie/image-optimizer-rust/ci.yml?branch=main&style=flat-square)](https://github.com/spatie/image-optimizer-rust/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](https://github.com/spatie/image-optimizer-rust/blob/main/LICENSE.md)
[![Total downloads](https://img.shields.io/crates/d/spatie-image-optimizer.svg?style=flat-square)](https://crates.io/crates/spatie-image-optimizer)

This package can optimize PNGs, JPGs, WEBPs, AVIFs, SVGs and GIFs by running them through a chain of various [image optimization tools](docs/which-tools-will-do-what.md). Here's how you can use it:

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

Here are some [example conversions](docs/example-conversions.md) that have been done by this package.

Install the [optimization tools](docs/installation-and-setup.md#optimization-tools) first. See [usage](docs/usage.md) to keep the original image or enable safe output.

This is the Rust version of [spatie/image-optimizer](https://github.com/spatie/image-optimizer).

## Support us

[<img src="https://spatie.be/github-ad-image/image-optimizer.jpg" alt="Support Spatie's open source work" width="419px" />](https://spatie.be/github-ad-click/image-optimizer)

We invest a lot of resources into creating [best in class open source packages](https://spatie.be/open-source). You can support us by [buying one of our paid products](https://spatie.be/open-source/support-us).

We highly appreciate you sending us a postcard from your hometown, mentioning which of our package(s) you are using. You'll find our address on [our contact page](https://spatie.be/about-us). We publish all received postcards on [our virtual postcard wall](https://spatie.be/open-source/postcards).

## Installation

You can install the package via Cargo. Add the crate to your `Cargo.toml`:

```toml
[dependencies]
spatie-image-optimizer = "1"
```

Requires Rust 1.89 or newer.

Install the [native optimization tools](docs/installation-and-setup.md) you need.
Missing binaries are logged and the chain continues by default, just like other optimizer failures.
Use `.throws(true)` to make failures abort instead.

## Usage

### Which tools will do what?

The package will automatically decide which tools to use on a particular image. See [which tools will do what](docs/which-tools-will-do-what.md) for the options used by each optimizer.

### Configuring the chain

See [usage](docs/usage.md) for timeouts, errors, custom chains and custom optimizers. [Safe output and reports](docs/safe-output-and-reports.md) are opt-in additions.

### Logging the optimization process

To verify what the package is doing you can set a logger. The package will write to log which optimizers are used, which commands are executed and their output. See [logging the optimization process](docs/logging-the-optimization-process.md).

### Example conversions

See [example conversions](https://spatie.be/docs/image-optimizer-rust/v1/example-conversions) for before and after images, measured file sizes and code to try the conversions yourself.

## Testing

Run from this directory:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
cargo test --locked --test native_tools -- --ignored --nocapture
cargo test --locked --test documentation -- --ignored
```

The last two commands require the native tools. The other tests do not require installed codecs. Native tests check that all six formats shrink, source files remain unchanged, and AVIF output can be decoded. They also check quality settings and cleanup after an encoder failure.

CI runs on Linux, macOS and Windows, checks Rust 1.89 on Linux, and runs rustfmt, Clippy and rustdoc. A separate Linux job runs the native and documentation tests with the tools installed.

Rustdoc compiles every Rust code block from this README and `docs/`. Setup uses hidden `# ` lines so the displayed examples stay close to PHP. Native examples use `no_run` in rustdoc; the `documentation` target executes those same self-contained examples with the tools installed.

## Changelog

Please see [CHANGELOG](CHANGELOG.md) for more information on what has changed recently.

## Contributing

See [CONTRIBUTING.md](https://github.com/spatie/image-optimizer-rust/blob/main/CONTRIBUTING.md) and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

Using an AI coding agent? Load Apollo's [rust-best-practices skill](https://github.com/apollographql/skills/tree/main/skills/rust-best-practices) when writing or reviewing code in this package.

## Security Vulnerabilities

Report vulnerabilities privately using the instructions in [SECURITY.md](https://github.com/spatie/image-optimizer-rust/blob/main/SECURITY.md).

Only local files are accepted. Native image tools process untrusted data in your process's security context; keep them updated and use operating-system isolation for untrusted uploads. Default in-place optimization is not transactional. An encoder failure can leave a partial output. Use `safe_output(true)` to work on a copy and publish only a smaller, error-free candidate. This is not a sandbox or an image-validity check. Use `optimize_to` when you also want to keep the source after successful optimization. Read the [full safety contract](docs/safe-output-and-reports.md#keeping-the-original-safe) before relying on replacement guarantees.

## Credits

Original PHP package by Freek Van der Herten and [contributors](https://github.com/spatie/image-optimizer/graphs/contributors).
Test images are copied from the PHP package; the photo is by Jeff Sheldon via Unsplash.

- [Freek Van der Herten](https://github.com/freekmurze)
- [All contributors](https://github.com/spatie/image-optimizer-rust/graphs/contributors)

## License

The MIT License (MIT). Please see [License File](LICENSE.md) for more information.
