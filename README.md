# Easily optimize images using Rust

This package can optimize PNGs, JPGs, WEBPs, AVIFs, SVGs and GIFs by running them through a chain of various [image optimization tools](docs/which-tools-will-do-what.md). Here's how you can use it:

```rust,ignore
use spatie_image_optimizer::OptimizerChainFactory;

let optimizer_chain = OptimizerChainFactory::create();

optimizer_chain.optimize(path_to_image)?;
```

The image at `path_to_image` will be overwritten by an optimized version which should be smaller. The package will automatically decide which tools to use on a particular image. Install the [binaries you need](docs/installation-and-setup.md#optimization-tools) first. Missing binaries are logged and the chain continues; they are not skipped by an installation check.

Here are some [example conversions](docs/example-conversions.md) that have been done by this package.

To keep the original image, use `optimize_to(path_to_image, path_to_output)`. To replace a file only when optimization succeeds and makes it smaller, enable [safe output](docs/usage.md#keeping-the-original-safe).

This is the Rust version of [spatie/image-optimizer](https://github.com/spatie/image-optimizer).

## Support us

Support our open source work by [buying one of our products](https://spatie.be/open-source/support-us).

## Installation

You can install the package via Cargo. Add this Git dependency to your `Cargo.toml`:

```toml
[dependencies]
spatie-image-optimizer = { git = "https://github.com/spatie/image-optimizer-rust", branch = "main" }
```

Requires Rust 1.89 or newer. The crate is not yet published to crates.io. Commit your application's `Cargo.lock` to keep the selected Git revision between builds.

Install the [native optimization tools](docs/installation-and-setup.md) you need.
Missing binaries are logged and the chain continues by default, just like other optimizer failures.
Use `.throws(true)` to make failures abort instead.

## Which tools will do what?

The [tool reference](docs/which-tools-will-do-what.md) lists the exact defaults for
JPEG, PNG, SVG, GIF, WebP and AVIF, including the AVIF decode/encode sequence.

## Usage

See [usage](docs/usage.md) for timeouts, errors, quality, custom chains and custom optimizers.
Binary commands use literal arguments without a shell. For example, use `["-q", "85"]`, not `["-q 85"]`.

The guide also covers [safe output](docs/usage.md#keeping-the-original-safe) and
[optimization reports](docs/usage.md#reading-optimization-reports), including what happens when
files grow, binaries are missing or encoders fail. The original methods still return `Result<(), Error>`;
`optimize_with_report` and `optimize_to_with_report` add sizes, timing and per-optimizer outcomes.

## Logging the optimization process

Use `.use_logger(TracingLogger)` or a custom logger. See [logging](docs/logging-the-optimization-process.md).

## Example conversions

See [example conversions](https://spatie.be/docs/image-optimizer-rust/v1/example-conversions) for before and after images, measured file sizes and code to try the conversions yourself.

## Changelog

Please see [CHANGELOG](CHANGELOG.md) for more information on what has changed recently.

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

The last two commands require the native tools. CI runs them in a separate Linux job.
The other tests do not require installed codecs.

The `documentation` target compiles and runs every Rust code block from this README and `docs/` with temporary images and application context. Rustdoc skips these fragments; this target executes them.

## Contributing

See [Spatie's contribution guidelines](https://github.com/spatie/.github/blob/main/CONTRIBUTING.md).
The [PHP and Rust API guide](docs/parity.md) shows the corresponding method names and differences to keep in mind.

## Security

Report security issues to [security@spatie.be](mailto:security@spatie.be).
Only local files are accepted. Native image tools process untrusted data in your process's security context;
keep them updated and use operating-system isolation for untrusted uploads. Default in-place optimization is not
transactional. An encoder failure can leave a partial output. Use `safe_output(true)` to work on a copy
and publish only a smaller, error-free candidate. This is not a sandbox or an image-validity check.
Use `optimize_to` when you also want to keep the source after successful optimization.

## Postcardware

If you use this package in production, we'd appreciate a postcard from your hometown.
Our address is Spatie, Kruikstraat 22, 2018 Antwerp, Belgium.

## Credits

Original PHP package by Freek Van der Herten and [contributors](https://github.com/spatie/image-optimizer/graphs/contributors).
Test images are copied from the PHP package; the photo is by Jeff Sheldon via Unsplash.

## License

MIT. See [LICENSE.md](LICENSE.md).
