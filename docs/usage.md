---
title: Usage
weight: 4
---

```rust,no_run
use spatie_image_optimizer::OptimizerChainFactory;

let chain = OptimizerChainFactory::create();
chain.optimize("photo.jpg")?;
chain.optimize_to("photo.jpg", "optimized.jpg")?;
# Ok::<(), spatie_image_optimizer::Error>(())
```

`optimize` overwrites in place. `optimize_to` copies the source before optimizing the copy.
Output parent directories must exist. Input and output aliases, including hard links, are rejected.
Neither operation guarantees a smaller image or atomic replacement. Non-image files pass through unchanged.

## Keeping the original safe

Enable `safe_output(true)` to optimize a working copy before replacing the destination:

```rust,no_run
use spatie_image_optimizer::{OptimizerChainFactory, OutputStatus};

let report = OptimizerChainFactory::create()
    .safe_output(true)
    .optimize_with_report("photo.jpg")?;

match report.status {
    OutputStatus::Written => println!("Saved {} bytes", report.bytes_saved()),
    OutputStatus::KeptOriginal => println!("No size reduction; kept the original"),
    OutputStatus::Discarded => eprintln!("An optimizer failed; kept the original"),
}
# Ok::<(), spatie_image_optimizer::Error>(())
```

The chain runs against a temporary copy in the destination's parent directory. It keeps the
filename and extension so extension-sensitive optimizers, including SVGO, still match.
The temporary directory uses mode `0o700` on Unix. On Windows it inherits the parent's ACL.
Use a trusted, writable parent directory with enough space for the copy and optimizer scratch files.

| Result | In-place optimization | Separate output |
| --- | --- | --- |
| All matching tools succeed and the candidate is smaller | Replace the original | Write the smaller candidate |
| Candidate is equal in size or larger, or no tools match | Leave the original untouched | Write a byte-for-byte copy of the source |
| Any optimizer fails and the error policy continues | Leave the original untouched | Leave an existing destination untouched; do not create a new one |
| Error policy aborts, or copying/publication fails | Return `Err` | Return `Err` |

Safe output does not change `throws` or `on_error`. A continued error still lets later tools run,
but the chain discards the whole candidate, even if a later tool succeeds. Missing binaries and
timeouts count as failures, not skips. With the default error policy, inspect the report for
`Discarded`; `Ok` alone means the chain completed, not that it published an optimized file.
Use `.throws(true)` if any optimizer failure should return `Err` immediately.

To preserve the source and use a separate destination:

```rust,no_run
use spatie_image_optimizer::OptimizerChainFactory;

let report = OptimizerChainFactory::create()
    .safe_output(true)
    .throws(true)
    .optimize_to_with_report("photo.jpg", "optimized.jpg")?;
println!("{:?}: {} bytes saved", report.status, report.bytes_saved());
# Ok::<(), spatie_image_optimizer::Error>(())
```

Successful publication uses a same-filesystem rename, not a copy over the destination. This gives
atomic replacement on filesystems that support it; the library does not `fsync` files or directories
and does not promise durability through power loss. Normal return and error unwinding clean up the
working directory. Forced process termination can leave it behind.

Destination symlinks, including broken ones, are rejected. Source/output aliases remain rejected
for separate output. In-place replacement creates a new file, so other hard links and already-open
handles continue to refer to the old file. The chain copies the destination's `std::fs::Permissions`
when replacing a file, or the source's permissions for a new output. Ownership, ACLs, extended
attributes and timestamps are not guaranteed to survive replacement. Read-only destinations may
prevent replacement, especially on Windows.

Custom optimizers and error callbacks receive the working path in safe mode; report paths always
identify the requested input and output. Optimizers that discover configuration relative to the
image's directory may need an explicit configuration path. Safe output trusts successful encoders;
it does not validate decoded pixels, preserve image metadata automatically, sandbox native programs,
or protect against another process changing paths during the operation.

The existing `optimize` and `optimize_to` methods also honor safe output, but discard the report.
Call `.safe_output(false)` to restore the default direct-write behavior.

## Reading optimization reports

Use `optimize_with_report` or `optimize_to_with_report` with any chain, with or without safe output:

```rust,no_run
use spatie_image_optimizer::{OptimizerChainFactory, OptimizerStatus};

let report = OptimizerChainFactory::create()
    .safe_output(true)
    .optimize_with_report("photo.jpg")?;

println!("Input: {} bytes; candidate: {:?}; elapsed: {:?}",
    report.original_bytes, report.candidate_bytes, report.elapsed);
for optimizer in &report.optimizers {
    match &optimizer.status {
        OptimizerStatus::Succeeded => println!("{}: completed in {:?}", optimizer.name, optimizer.elapsed),
        OptimizerStatus::Skipped => println!("{}: unsupported format", optimizer.name),
        OptimizerStatus::Failed(message) => eprintln!("{}: {message}", optimizer.name),
    }
}
# Ok::<(), spatie_image_optimizer::Error>(())
```

- `input` and `output` are the requested local paths. File URLs are resolved; paths are not canonicalized.
- `original_bytes` is the input size before the chain runs.
- `candidate_bytes` is the working file size after the chain, **before** accepting or discarding it.
  It can be larger than the original and is `None` if a failed optimizer left no readable candidate.
  For `KeptOriginal` or `Discarded`, it is not the destination's final size.
- `bytes_saved()` counts a published reduction. It returns zero for growth, retained originals,
  missing candidates and discarded candidates. It does not compare against a pre-existing output file.
- `optimizers` includes every configured entry in execution order. `Skipped` means `can_handle`
  returned false; `Succeeded` means the optimizer returned `Ok`, not necessarily that it saved bytes.
  `Failed(String)` contains the continued error's display text, not a typed error.
- `elapsed` covers preparation, execution and publication. Per-tool durations include support checks.

Without safe output, `Written` does not promise smaller, valid or complete output: continued errors
may have left partial changes. For typed process errors, use `on_error` or `.throws(true)`.
An aborted chain returns `Err` rather than a partial report.

## Setting a timeout

```rust,no_run
use spatie_image_optimizer::OptimizerChainFactory;
use std::time::Duration;

OptimizerChainFactory::create()
    .set_timeout(Duration::from_secs(10))
    .optimize("photo.jpg")?;
# Ok::<(), spatie_image_optimizer::Error>(())
```

The default is 60 seconds per optimizer, not per chain. Zero disables binary timeouts.
A timed-out binary is killed and reaped. The library does not kill independently spawned descendants.
Custom self-handling optimizers are responsible for their own timeout enforcement.

## Handling errors

By default optimizer failures, missing binaries and timeouts are logged and the chain continues.
Validation and output-copy errors always return an `Err`. Rust panics are not caught.

```rust,no_run
use spatie_image_optimizer::{Error, OptimizerChainFactory};

OptimizerChainFactory::create().throws(true).optimize("photo.jpg")?;
OptimizerChainFactory::create().throws(false).optimize("photo.jpg")?;

OptimizerChainFactory::create()
    .on_error(|error, optimizer, image| {
        eprintln!("{} failed for {}: {error}", optimizer.name(), image.path().display());
        Ok(())
    })
    .optimize("photo.jpg")?;

let abort = OptimizerChainFactory::create()
    .on_error(|_, _, _| Err(Error::Custom("aborted".into())));
# Ok::<(), Error>(())
```

The handler receives the error, optimizer and image. Return `Ok(())` to continue or `Err(...)` to abort.
Calling `throws` replaces a custom handler. `Error::Process` includes the exit code, stdout and stderr.

## Creating your own optimization chains

```rust,no_run
use spatie_image_optimizer::{OptimizerChain, Tool};

let chain = OptimizerChain::new()
    .add_optimizer(Tool::Jpegoptim.with_options(["--strip-all", "--all-progressive"]))
    .add_optimizer(Tool::Pngquant.with_options(["--force"]).set_binary_path("/opt/image-tools"));
chain.optimize("photo.jpg")?;
# Ok::<(), spatie_image_optimizer::Error>(())
```

`set_optimizers(Vec<Box<dyn Optimizer>>)` replaces the chain. `get_optimizers()` exposes its ordered entries.
Use `set_options` to replace arguments, `set_binary_name` to select another executable and
`set_decode_binary_name` to override the AVIF decoder. Each option is one literal argument; no shell
expansion, interpolation, pipelines or redirection occurs. A flag with a value takes two items,
such as `["-q", "85"]`. A single option like `"--quality=85"` remains one item.

For a default chain with a different quality use `OptimizerChainFactory::with_quality(50)?`.

## Writing custom optimizers

Use `std::process::Command` and `run_command` for a custom binary. Implement `can_handle` to choose
supported images. The chain calls `optimize` only for matching files.

```rust,no_run
use spatie_image_optimizer::{Error, Image, Logger, Optimizer, OptimizerChain, run_command};
use std::{process::Command, time::Duration};

struct MyOptimizer;
impl Optimizer for MyOptimizer {
    fn name(&self) -> &str { "my-optimizer" }
    fn can_handle(&self, image: &Image) -> bool { image.mime().is_ok_and(|mime| mime == "image/png") }
    fn optimize(&self, image: &Image, logger: &dyn Logger, timeout: Duration) -> Result<(), Error> {
        run_command(Command::new("my-optimizer").arg(image.path()), timeout, logger)
    }
}

OptimizerChain::new().add_optimizer(MyOptimizer).optimize("logo.png")?;
# Ok::<(), Error>(())
```

## Writing an optimizer without a binary

The same `Optimizer` trait handles APIs and in-process code; no empty binary methods or base class are needed.
Write the optimized bytes to `image.path()` and return `Result<(), Error>`. Set your network client's
timeout from the supplied duration. Keep scratch files in RAII owners such as `tempfile::NamedTempFile`
so they are removed before the error callback runs. Self-handling failures use the same error policy as binaries.
