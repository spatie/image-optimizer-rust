---
title: Usage
weight: 5
---

This is the default way to use the package:

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

The image at `path_to_image` will be overwritten by an optimized version which should be smaller.

The package will automatically decide which tools to use on a particular image. Install the [optimization tools](installation-and-setup.md#optimization-tools) first; missing binaries are logged failures, not skipped by an installation check.

To keep the original image, you can pass through a second argument to `optimize_to`:

```rust,no_run
# fn main() -> Result<(), spatie_image_optimizer::Error> {
# let directory = tempfile::tempdir()?;
# let source = directory.path().join("image.jpg");
# let output = directory.path().join("optimized.jpg");
# std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/image.jpg"), &source)?;
# let path_to_image = source.as_path();
# let path_to_output = output.as_path();
use spatie_image_optimizer::OptimizerChainFactory;

let optimizer_chain = OptimizerChainFactory::create();

optimizer_chain.optimize_to(path_to_image, path_to_output)?;
# assert_eq!(spatie_image_optimizer::Image::new(&output)?.mime()?, "image/jpeg");
# assert_eq!(std::fs::read(&source)?, std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/image.jpg"))?);
# Ok(())
# }
```

In that example the package won't touch `path_to_image` and write an optimized version to `path_to_output`.

For guarantees about replacing the original, see [keeping the original safe](#keeping-the-original-safe).

<a id="setting-a-timeout"></a>

## Setting a timeout

You can set the maximum of time in seconds that each individual optimizer in a chain can use by calling `set_timeout`:

```rust,no_run
# fn main() -> Result<(), spatie_image_optimizer::Error> {
# let directory = tempfile::tempdir()?;
# let source = directory.path().join("image.jpg");
# std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/image.jpg"), &source)?;
# let path_to_image = source.as_path();
# let optimizer_chain = spatie_image_optimizer::OptimizerChainFactory::create();
use std::time::Duration;

optimizer_chain
    .set_timeout(Duration::from_secs(10))
    .optimize(path_to_image)?;
# Ok(())
# }
```

In this example each optimizer in the chain will get a maximum 10 seconds to do its job.

<a id="handling-errors"></a>

## Handling errors

By default, when an optimizer fails (for example its binary exits with a non-zero status), the failure is logged and the chain simply continues with the next optimizer. If you'd rather be notified, or abort the whole chain, use `throws`.

Call `throws(true)` to make the chain return the failure and stop:

```rust,no_run
# fn main() -> Result<(), spatie_image_optimizer::Error> {
# let directory = tempfile::tempdir()?;
# let source = directory.path().join("image.jpg");
# std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/image.jpg"), &source)?;
# let path_to_image = source.as_path();
# let optimizer_chain = spatie_image_optimizer::OptimizerChainFactory::create();
optimizer_chain
    .throws(true)
    .optimize(path_to_image)?;
# Ok(())
# }
```

Pass `false` to keep the default "log and continue" behaviour explicitly:

```rust,no_run
# fn main() -> Result<(), spatie_image_optimizer::Error> {
# let directory = tempfile::tempdir()?;
# let source = directory.path().join("image.jpg");
# std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/image.jpg"), &source)?;
# let path_to_image = source.as_path();
# let optimizer_chain = spatie_image_optimizer::OptimizerChainFactory::create();
optimizer_chain
    .throws(false)
    .optimize(path_to_image)?;
# Ok(())
# }
```

Or pass a closure to decide for yourself. The handler receives the error, the optimizer that failed and the image being optimized. Return `Ok(())` to continue with the next optimizer, or return `Err(...)` to abort the chain:

```rust,no_run
# fn main() -> Result<(), spatie_image_optimizer::Error> {
# let directory = tempfile::tempdir()?;
# let source = directory.path().join("image.jpg");
# std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/image.jpg"), &source)?;
# let path_to_image = source.as_path();
# let optimizer_chain = spatie_image_optimizer::OptimizerChainFactory::create();
# fn report(error: &spatie_image_optimizer::Error) { eprintln!("{error}"); }
optimizer_chain
    .on_error(|exception, optimizer, image| {
        report(exception);

        // Return Ok(()) to continue the chain, or Err(...) to abort it.
        Ok(())
    })
    .optimize(path_to_image)?;
# Ok(())
# }
```

Any error raised while applying an optimizer, most notably a process timeout when an optimizer exceeds its [timeout](#setting-a-timeout), flows through this same mechanism. With the default (or `throws(false)`) it is caught and the chain continues; with `throws(true)` or a closure you get to inspect and decide what to do with it.

Validation and filesystem errors still return `Err`. Calling `throws` replaces a custom handler. A timeout returns `Error::Timeout`; `Error::Process` includes the exit code, stdout and stderr. Rust panics are not caught. `report` above is your application's error-reporting function.

## Creating your own optimization chains

If you want to customize the chain of optimizers you can do so by adding `Optimizer`s manually to an `OptimizerChain`.

Here's an example where we only want `jpegoptim` and `pngquant` to be used:

```rust
# fn main() -> Result<(), spatie_image_optimizer::Error> {
use spatie_image_optimizer::{OptimizerChain, Tool};

let optimizer_chain = OptimizerChain::new()
    .add_optimizer(Tool::Jpegoptim.with_options(["--strip-all", "--all-progressive"]))
    .add_optimizer(Tool::Pngquant.with_options(["--force"]));
# Ok(())
# }
```

Notice that you can pass the options an optimizer should use to `with_options`.

## Writing custom optimizers

Want to use another command line utility to optimize your images? No problem. Just write your own optimizer. An optimizer is any type that implements the `Optimizer` trait:

```rust
# fn main() -> Result<(), spatie_image_optimizer::Error> {
use spatie_image_optimizer::{Error, Image, Logger};
use std::time::Duration;

pub trait Optimizer: Send + Sync {
    fn name(&self) -> &str;
    fn can_handle(&self, image: &Image) -> bool;
    fn optimize(&self, image: &Image, logger: &dyn Logger, timeout: Duration) -> Result<(), Error>;
}
# Ok(())
# }
```

If you want to view an example implementation take a look at [the existing optimizers](https://github.com/spatie/image-optimizer-rust/blob/main/src/optimizers.rs) shipped with this package.

You can easily add your optimizer by using the `add_optimizer` method on an `OptimizerChain`:

```rust,no_run
# fn main() -> Result<(), spatie_image_optimizer::Error> {
# let directory = tempfile::tempdir()?;
# let source = directory.path().join("image.jpg");
# std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/image.jpg"), &source)?;
# let path_to_image = source.as_path();
# struct YourCustomOptimizer;
# impl spatie_image_optimizer::Optimizer for YourCustomOptimizer {
#     fn name(&self) -> &str { "custom-jpegoptim" }
#     fn can_handle(&self, image: &spatie_image_optimizer::Image) -> bool {
#         spatie_image_optimizer::Tool::Jpegoptim.with_options(["--strip-all"]).can_handle(image)
#     }
#     fn optimize(&self, image: &spatie_image_optimizer::Image, logger: &dyn spatie_image_optimizer::Logger, timeout: std::time::Duration) -> Result<(), spatie_image_optimizer::Error> {
#         spatie_image_optimizer::Tool::Jpegoptim.with_options(["--strip-all"]).optimize(image, logger, timeout)
#     }
# }
use spatie_image_optimizer::OptimizerChainFactory;

let optimizer_chain = OptimizerChainFactory::create();

optimizer_chain
    .add_optimizer(YourCustomOptimizer)
    .optimize(path_to_image)?;
# Ok(())
# }
```

## Writing an optimizer without a binary

Sometimes an optimizer has no binary and no shell command to run, for example one that sends the image to an external optimization API. For these cases implement the same `Optimizer` trait. The chain delegates execution to your `optimize` method rather than building and running a process:

```rust
# fn main() -> Result<(), spatie_image_optimizer::Error> {
use spatie_image_optimizer::{Error, Image, Logger, Optimizer};
use std::time::Duration;

struct ApiOptimizer;

impl Optimizer for ApiOptimizer {
    fn name(&self) -> &str {
        "api-optimizer"
    }

    fn can_handle(&self, image: &Image) -> bool {
        image.mime().is_ok_and(|mime| mime == "image/jpeg")
    }

    fn optimize(&self, image: &Image, logger: &dyn Logger, timeout: Duration) -> Result<(), Error> {
        // Optimize image.path() however you like, e.g. by calling an API,
        // and write the optimized bytes back to that path. Return Err(...) on failure.
        // The chain's logger and timeout are passed in for your implementation to use.
        Ok(())
    }
}
# Ok(())
# }
```

Add it to a chain with `add_optimizer` just like any other optimizer. Failures are governed by [`throws`](#handling-errors) in exactly the same way as binary optimizers: by default the failure is logged and the chain continues, while `throws(true)` (or a closure) lets you abort or handle it.

Set your network client's timeout from the supplied duration. Custom optimizers own and clean up their scratch files. Commands use literal arguments, not a shell. Pass `["-q", "85"]`, not `["-q 85"]`. `set_options` replaces arguments, `set_binary_path` selects their directory, `set_binary_name` overrides an executable and `set_decode_binary_name` overrides the AVIF decoder. Zero disables binary timeouts. A timed-out direct child is killed and reaped; independently spawned descendants are not.

<a id="keeping-the-original-safe"></a>

## Keeping the original safe

Direct writes can leave partial or larger files. Use `safe_output(true)` for safer replacement; read the [examples and safety contract](safe-output-and-reports.md#keeping-the-original-safe). This is not a sandbox or an image-validity check.

<a id="reading-optimization-reports"></a>

## Reading optimization reports

Use `optimize_with_report` or `optimize_to_with_report` for sizes, timing and per-optimizer outcomes. See the [report example and field reference](safe-output-and-reports.md#reading-optimization-reports); check `report.status` before assuming a file was published.
