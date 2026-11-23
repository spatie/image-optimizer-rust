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

The image at `path_to_image` will be overwritten by an optimized version which should be smaller. The package will automatically decide which tools to use on a particular image. Make sure the [binaries you need](installation-and-setup.md#optimization-tools) are installed.

To keep the original image, you can pass through a second argument to `optimize_to`:

```rust,ignore
use spatie_image_optimizer::OptimizerChainFactory;

let optimizer_chain = OptimizerChainFactory::create();

optimizer_chain.optimize_to(path_to_image, path_to_output)?;
```

In that example the package won't touch `path_to_image` and will write an optimized version to `path_to_output`.

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

In this example each optimizer in the chain gets a maximum of 10 seconds to do its job.

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

Or pass a closure to decide for yourself. Use `on_error` for this in Rust. The handler receives the error, the optimizer that failed and the image being optimized. Return `Ok(())` to continue with the next optimizer, or return `Err(...)` to abort the chain:

```rust,ignore
optimizer_chain
    .on_error(|exception, optimizer, image| {
        report(exception);

        // Return Ok(()) to continue the chain, or Err(...) to abort it.
        Ok(())
    })
    .optimize(path_to_image)?;
```

Here, `report` is your application's error-reporting function. Any error raised while applying an optimizer, most notably an `Error::Timeout` when an optimizer exceeds its [timeout](#setting-a-timeout), flows through this same mechanism. With the default (or `throws(false)`) it is logged and the chain continues; with `throws(true)` or a closure you get to inspect and decide what to do with it.

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

If you want to view an example implementation, take a look at [the existing optimizers](https://github.com/spatie/image-optimizer-rust/blob/main/src/optimizers.rs) shipped with this package. Binary optimizers can use `run_command` to run a `Command` with logging and a timeout.

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

Add it to a chain with `add_optimizer` just like any other optimizer. Failures are governed by [`throws`](#handling-errors) in exactly the same way as binary optimizers: by default the failure is logged and the chain continues, while `throws(true)` (or a closure) lets you abort or handle it. Set your network client's timeout from the supplied duration.

## Chain configuration details

The default timeout is 60 seconds per optimizer, not per chain. Zero disables binary timeouts. A timed-out binary is killed and reaped, but independently spawned descendants are not. Custom self-handling optimizers must enforce their own timeout. Keep scratch files in owners such as `tempfile::NamedTempFile` so they are removed before the error callback runs.

Calling `throws` replaces a custom handler. `Error::Process` includes the exit code, stdout and stderr. Validation and output-copy errors always return `Err`; Rust panics are not caught.

Options are literal arguments. Use `["-q", "85"]`, not `["-q 85"]`; no shell expansion, pipelines or redirection occurs. Use `set_options` to replace arguments, `set_binary_path` to select a directory containing the binaries, `set_binary_name` to select another executable, and `set_decode_binary_name` to override the AVIF decoder. `set_optimizers(Vec<Box<dyn Optimizer>>)` replaces the chain; `get_optimizers()` gives you its entries in execution order.

For a default chain with a different quality, use `OptimizerChainFactory::with_quality(50)?`.

<a id="keeping-the-original-safe"></a>

## Keeping the original safe

Enable `safe_output(true)` to optimize a working copy before replacing the destination:

```rust,ignore
use spatie_image_optimizer::{OptimizerChainFactory, OutputStatus};

let report = OptimizerChainFactory::create()
    .safe_output(true)
    .optimize_with_report(path_to_image)?;

match report.status {
    OutputStatus::Written => println!("Saved {} bytes", report.bytes_saved()),
    OutputStatus::KeptOriginal => println!("No size reduction; kept the original"),
    OutputStatus::Discarded => eprintln!("An optimizer failed; kept the original"),
}
```

Neither `optimize` nor `optimize_to` guarantees a smaller image in the default direct-write mode. A failed optimizer can leave partial changes. Non-image files pass through unchanged. For separate output, the output directory must already exist, and the output must not refer to the source, including through a hard link.

The chain works on a temporary copy in the destination's parent directory, preserving the filename and extension for tools such as SVGO. Use a trusted, writable parent with enough space for that copy and optimizer scratch files. The temporary directory uses mode `0o700` on Unix and inherits the parent's ACL on Windows.

| Result | In-place optimization | Separate output |
| --- | --- | --- |
| All matching tools succeed and the candidate is smaller | Replace the original | Write the smaller candidate |
| Candidate is equal in size or larger, or no tools match | Leave the original untouched | Write a byte-for-byte copy of the source |
| Any optimizer fails and the error policy continues | Leave the original untouched | Leave an existing destination untouched; do not create a new one |
| Error policy aborts, or copying/publication fails | Return `Err` | Return `Err` |

Safe output does not change `throws` or `on_error`. Continued errors let later tools run but discard the whole candidate, even if a later tool succeeds. Missing binaries and timeouts are failures, not skips. Check for `Discarded`: `Ok` alone does not mean an optimized file was published. Use `.throws(true)` to return `Err` on any optimizer failure.

To preserve the source and use a separate destination:

```rust,ignore
use spatie_image_optimizer::OptimizerChainFactory;

let report = OptimizerChainFactory::create()
    .safe_output(true)
    .throws(true)
    .optimize_to_with_report(path_to_image, path_to_output)?;
println!("{:?}: {} bytes saved", report.status, report.bytes_saved());
```

Publication uses a same-filesystem rename, atomic where the filesystem supports it. There is no `fsync` or power-loss durability guarantee. Normal return and error unwinding remove the working directory; forced termination may leave it behind.

Destination symlinks, including broken ones, and separate-output source aliases are rejected. Replacement creates a new file; other hard links and open handles still refer to the old one. Existing destinations retain their `std::fs::Permissions`; new outputs copy the source's permissions. Ownership, ACLs, extended attributes and timestamps may change. Read-only destinations can prevent replacement, especially on Windows.

Custom optimizers and callbacks receive the working path; reports identify the requested paths. Configuration discovery relative to the image may require an explicit configuration path. Safe output trusts encoders: it does not validate pixels, automatically preserve metadata, sandbox programs or prevent concurrent path changes.

`optimize` and `optimize_to` also honor safe output but discard the report. Call `.safe_output(false)` to restore direct writes.

<a id="reading-optimization-reports"></a>

## Reading optimization reports

Use `optimize_with_report` or `optimize_to_with_report` with any chain, with or without safe output:

```rust,ignore
use spatie_image_optimizer::{OptimizerChainFactory, OptimizerStatus};

let report = OptimizerChainFactory::create()
    .safe_output(true)
    .optimize_with_report(path_to_image)?;

println!("Input: {} bytes; candidate: {:?}; elapsed: {:?}",
    report.original_bytes, report.candidate_bytes, report.elapsed);
for optimizer in &report.optimizers {
    match &optimizer.status {
        OptimizerStatus::Succeeded => println!("{}: completed in {:?}", optimizer.name, optimizer.elapsed),
        OptimizerStatus::Skipped => println!("{}: unsupported format", optimizer.name),
        OptimizerStatus::Failed(message) => eprintln!("{}: {message}", optimizer.name),
    }
}
```

- `input` and `output` are requested local paths. File URLs are resolved, not canonicalized.
- `original_bytes` is the input size before optimization.
- `candidate_bytes` is the working size **before** accepting or discarding it, not necessarily the final destination size. It may be larger, or `None` if failure left no readable candidate.
- `bytes_saved()` counts published reduction against the source, not a pre-existing output. Growth, retained originals and missing or discarded candidates return zero.
- `optimizers` lists all entries in order. `Skipped` means `can_handle` returned false; `Succeeded` means `Ok`, not necessarily smaller output. `Failed(String)` holds the continued error's display text, not a typed error.
- `elapsed` includes preparation, execution and publication; per-tool times include support checks.

Without safe output, `Written` can mean larger, invalid or partial output after a continued error. Use `on_error` or `.throws(true)` for typed process errors. Aborted chains return `Err`, not partial reports.
