---
title: Usage
weight: 5
---

This is the default way to use the package:

```rust,ignore
use spatie_image_optimizer::OptimizerChainFactory;

let optimizer_chain = OptimizerChainFactory::create();

optimizer_chain.optimize(path_to_image)?;
```

The image at `path_to_image` will be overwritten by an optimized version which should be smaller.

The package will automatically decide which tools to use on a particular image. Install the [optimization tools](installation-and-setup.md#optimization-tools) first; missing binaries are logged failures, not skipped by an installation check.

To keep the original image, you can pass through a second argument to `optimize_to`:

```rust,ignore
use spatie_image_optimizer::OptimizerChainFactory;

let optimizer_chain = OptimizerChainFactory::create();

optimizer_chain.optimize_to(path_to_image, path_to_output)?;
```

In that example the package won't touch `path_to_image` and write an optimized version to `path_to_output`.

For guarantees about replacing the original, see [keeping the original safe](#keeping-the-original-safe).

<a id="setting-a-timeout"></a>

## Setting a timeout

You can set the maximum of time in seconds that each individual optimizer in a chain can use by calling `set_timeout`:

```rust,ignore
use std::time::Duration;

optimizer_chain
    .set_timeout(Duration::from_secs(10))
    .optimize(path_to_image)?;
```

In this example each optimizer in the chain will get a maximum 10 seconds to do its job.

<a id="handling-errors"></a>

## Handling errors

By default, when an optimizer fails (for example its binary exits with a non-zero status), the failure is logged and the chain simply continues with the next optimizer. If you'd rather be notified, or abort the whole chain, use `throws`.

Call `throws(true)` to make the chain return the failure and stop:

```rust,ignore
optimizer_chain
    .throws(true)
    .optimize(path_to_image)?;
```

Pass `false` to keep the default "log and continue" behaviour explicitly:

```rust,ignore
optimizer_chain
    .throws(false)
    .optimize(path_to_image)?;
```

Or pass a closure to decide for yourself. The handler receives the error, the optimizer that failed and the image being optimized. Return `Ok(())` to continue with the next optimizer, or return `Err(...)` to abort the chain:

```rust,ignore
optimizer_chain
    .on_error(|exception, optimizer, image| {
        report(exception);

        // Return Ok(()) to continue the chain, or Err(...) to abort it.
        Ok(())
    })
    .optimize(path_to_image)?;
```

Any error raised while applying an optimizer, most notably a process timeout when an optimizer exceeds its [timeout](#setting-a-timeout), flows through this same mechanism. With the default (or `throws(false)`) it is caught and the chain continues; with `throws(true)` or a closure you get to inspect and decide what to do with it.

Validation and filesystem errors still return `Err`. See the [Rust error reference](https://github.com/spatie/image-optimizer-rust/blob/main/PARITY.md#chain-configuration-and-errors) for timeout limits, callback replacement and process diagnostics. `report` above is your application's error-reporting function.

## Creating your own optimization chains

If you want to customize the chain of optimizers you can do so by adding `Optimizer`s manually to an `OptimizerChain`.

Here's an example where we only want `jpegoptim` and `pngquant` to be used:

```rust,ignore
use spatie_image_optimizer::{OptimizerChain, Tool};

let optimizer_chain = OptimizerChain::new()
    .add_optimizer(Tool::Jpegoptim.with_options(["--strip-all", "--all-progressive"]))
    .add_optimizer(Tool::Pngquant.with_options(["--force"]));
```

Notice that you can pass the options an optimizer should use to `with_options`.

## Writing custom optimizers

Want to use another command line utility to optimize your images? No problem. Just write your own optimizer. An optimizer is any type that implements the `Optimizer` trait:

```rust,ignore
use spatie_image_optimizer::{Error, Image, Logger};
use std::time::Duration;

pub trait Optimizer: Send + Sync {
    fn name(&self) -> &str;
    fn can_handle(&self, image: &Image) -> bool;
    fn optimize(&self, image: &Image, logger: &dyn Logger, timeout: Duration) -> Result<(), Error>;
}
```

If you want to view an example implementation take a look at [the existing optimizers](https://github.com/spatie/image-optimizer-rust/blob/main/src/optimizers.rs) shipped with this package.

You can easily add your optimizer by using the `add_optimizer` method on an `OptimizerChain`:

```rust,ignore
use spatie_image_optimizer::OptimizerChainFactory;

let optimizer_chain = OptimizerChainFactory::create();

optimizer_chain
    .add_optimizer(YourCustomOptimizer)
    .optimize(path_to_image)?;
```

## Writing an optimizer without a binary

Sometimes an optimizer has no binary and no shell command to run, for example one that sends the image to an external optimization API. For these cases implement the same `Optimizer` trait. The chain delegates execution to your `optimize` method rather than building and running a process:

```rust,ignore
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
```

Add it to a chain with `add_optimizer` just like any other optimizer. Failures are governed by [`throws`](#handling-errors) in exactly the same way as binary optimizers: by default the failure is logged and the chain continues, while `throws(true)` (or a closure) lets you abort or handle it.

Set your network client's timeout from the supplied duration. Custom optimizers own and clean up their scratch files. Commands use literal arguments, not a shell; see the [configuration reference](https://github.com/spatie/image-optimizer-rust/blob/main/PARITY.md#chain-configuration-and-errors).

<a id="keeping-the-original-safe"></a>

## Keeping the original safe

Direct writes can leave partial or larger files. Use `safe_output(true)` for safer replacement; read the [examples and safety contract](parity.md#keeping-the-original-safe). This is not a sandbox or an image-validity check.

<a id="reading-optimization-reports"></a>

## Reading optimization reports

Use `optimize_with_report` or `optimize_to_with_report` for sizes, timing and per-optimizer outcomes. See the [report example and field reference](parity.md#reading-optimization-reports); check `report.status` before assuming a file was published.
