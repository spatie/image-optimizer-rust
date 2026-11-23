# Easily optimize images using Rust

A standalone Rust port of [spatie/image-optimizer](https://github.com/spatie/image-optimizer).
Optimize JPEG, PNG, SVG, GIF, WebP and AVIF files with an ordered chain of native tools.

```rust,no_run
use spatie_image_optimizer::OptimizerChainFactory;

OptimizerChainFactory::create().optimize("photo.jpg")?;
OptimizerChainFactory::create().optimize_to("photo.jpg", "optimized.jpg")?;
# Ok::<(), spatie_image_optimizer::Error>(())
```

For new integrations, use safe output to keep the original on failures or when the candidate is not smaller:

```rust,no_run
use spatie_image_optimizer::{OptimizerChainFactory, OutputStatus};

let report = OptimizerChainFactory::create()
    .safe_output(true)
    .optimize_with_report("photo.jpg")?;

if report.status == OutputStatus::Discarded {
    eprintln!("Optimization failed; the original is unchanged");
} else {
    println!("Saved {} bytes in {:?}", report.bytes_saved(), report.elapsed);
}
# Ok::<(), spatie_image_optimizer::Error>(())
```

## Support us

Support our open source work by [buying one of our products](https://spatie.be/open-source/support-us).

## Installation

Requires Rust 1.89 or newer. This crate is not published; use a local path dependency:

```toml
[dependencies]
spatie-image-optimizer = { path = "../rust-image-optimizer" }
```

Install the [native optimization tools](docs/installation-and-setup.md) you need.
Missing binaries are logged and skipped by default, just like other optimizer failures.
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

See [example conversions](docs/example-conversions.md) for reproducible native-tool tests.

## Testing

Run from this directory:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
cargo test --locked --test native_tools -- --ignored --nocapture
```

The last command requires all native tools. CI runs it in a separate Linux job.
The other tests do not require installed codecs.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Contributing

See [Spatie's contribution guidelines](https://github.com/spatie/.github/blob/main/CONTRIBUTING.md).
The [parity reference](docs/parity.md) records the PHP-to-Rust API and deliberate safety differences.

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
