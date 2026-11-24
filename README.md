# Easily optimize images using Rust

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

We invest a lot of resources into creating [best in class open source packages](https://spatie.be/open-source). You can support us by [buying one of our paid products](https://spatie.be/open-source/support-us).

We highly appreciate you sending us a postcard from your hometown, mentioning which of our package(s) you are using. You'll find our address on [our contact page](https://spatie.be/about-us). We publish all received postcards on [our virtual postcard wall](https://spatie.be/open-source/postcards).

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

The package will automatically decide which tools to use on a particular image. See [which tools will do what](docs/which-tools-will-do-what.md) for the options used by each optimizer.

## Usage

See [usage](docs/usage.md) for timeouts, errors, custom chains and custom optimizers. The [PHP and Rust API reference](PARITY.md) covers quality settings, literal command arguments and Rust-specific behavior. [Safe output and reports](docs/parity.md) are opt-in additions.

## Logging the optimization process

To verify what the package is doing you can set a logger. The package will write to log which optimizers are used, which commands are executed and their output. See [logging the optimization process](docs/logging-the-optimization-process.md).

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

The last two commands require the native tools. The other tests do not require installed codecs. Native tests check that all six formats shrink, source files remain unchanged, and AVIF output can be decoded. They also check quality settings and cleanup after an encoder failure.

CI runs on Linux, macOS and Windows, checks Rust 1.89 on Linux, and runs rustfmt, Clippy and rustdoc. A separate Linux job runs the native and documentation tests with the tools installed.

Rustdoc compiles every Rust code block from this README and `docs/`. Setup uses hidden `# ` lines so the displayed examples stay close to PHP. Native examples use `no_run` in rustdoc; the `documentation` target executes those same self-contained examples with the tools installed.

## Contributing

Please see [CONTRIBUTING](https://github.com/spatie/.github/blob/main/CONTRIBUTING.md) for details.

## Security

If you've found a bug regarding security please mail [security@spatie.be](mailto:security@spatie.be) instead of using the issue tracker.

Only local files are accepted. Native image tools process untrusted data in your process's security context; keep them updated and use operating-system isolation for untrusted uploads. Default in-place optimization is not transactional. An encoder failure can leave a partial output. Use `safe_output(true)` to work on a copy and publish only a smaller, error-free candidate. This is not a sandbox or an image-validity check. Use `optimize_to` when you also want to keep the source after successful optimization. Read the [full safety contract](PARITY.md#keeping-the-original-safe) before relying on replacement guarantees.

## Postcardware

You're free to use this package (it's [MIT-licensed](LICENSE.md)), but if it makes it to your production environment we highly appreciate you sending us a postcard from your hometown, mentioning which of our package(s) you are using.

Our address is: Spatie, Kruikstraat 22, 2018 Antwerp, Belgium.

We publish all received postcards [on our company website](https://spatie.be/open-source/postcards).

## Credits

Original PHP package by Freek Van der Herten and [contributors](https://github.com/spatie/image-optimizer/graphs/contributors).
Test images are copied from the PHP package; the photo is by Jeff Sheldon via Unsplash.

## License

The MIT License (MIT). Please see [License File](LICENSE.md) for more information.
