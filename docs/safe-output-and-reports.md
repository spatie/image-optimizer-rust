---
title: Safe output and reports
weight: 10
---

Safe output optimizes a working copy and only publishes a smaller, error-free candidate. Reports expose sizes, timing and individual optimizer outcomes.

<a id="keeping-the-original-safe"></a>

## Keeping the original safe

Enable `safe_output(true)` to optimize a working copy before replacing the destination:

```rust,no_run
# fn main() -> Result<(), spatie_image_optimizer::Error> {
# let directory = tempfile::tempdir()?;
# let source = directory.path().join("image.jpg");
# std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/image.jpg"), &source)?;
# let path_to_image = source.as_path();
use spatie_image_optimizer::{OptimizerChainFactory, OutputStatus};

let report = OptimizerChainFactory::create()
    .safe_output(true)
    .optimize_with_report(path_to_image)?;

match report.status {
    OutputStatus::Written => println!("Saved {} bytes", report.bytes_saved()),
    OutputStatus::KeptOriginal => println!("No size reduction; kept the original"),
    OutputStatus::Discarded => eprintln!("An optimizer failed; kept the original"),
}
# Ok(())
# }
```

To preserve the source and use a separate destination:

```rust,no_run
# fn main() -> Result<(), spatie_image_optimizer::Error> {
# let directory = tempfile::tempdir()?;
# let source = directory.path().join("image.jpg");
# let output = directory.path().join("optimized.jpg");
# std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/image.jpg"), &source)?;
# let path_to_image = source.as_path();
# let path_to_output = output.as_path();
use spatie_image_optimizer::OptimizerChainFactory;

let report = OptimizerChainFactory::create()
    .safe_output(true)
    .throws(true)
    .optimize_to_with_report(path_to_image, path_to_output)?;
println!("{:?}: {} bytes saved", report.status, report.bytes_saved());
# assert_eq!(spatie_image_optimizer::Image::new(&output)?.mime()?, "image/jpeg");
# assert_eq!(std::fs::read(&source)?, std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/image.jpg"))?);
# Ok(())
# }
```

The working copy lives in a private temporary directory beside the destination. Use a trusted, writable parent directory with enough space for the copy and encoder scratch files. Destination symlinks and separate-output source aliases, including hard links, are rejected.

If the candidate is not smaller or no tools match, in-place optimization keeps the original and separate-output optimization writes a copy of the source. If any optimizer fails under a continuing error policy, the entire candidate is discarded and an existing destination stays untouched. Check `OutputStatus::Discarded`; `Ok` alone does not mean a file was published. Use `throws(true)` to abort on optimizer failures.

Publication uses a same-filesystem rename, atomic where supported. Existing destinations retain their `std::fs::Permissions`; new outputs copy the source permissions. Ownership, ACLs, extended attributes and timestamps may change. Other hard links and open handles still refer to the old file. There is no `fsync` or power-loss durability guarantee. Forced termination may leave scratch files behind.

Custom optimizers receive the working path. Supply explicit configuration paths when an encoder discovers configuration relative to its input. This is not a sandbox or an image-validity check. Keep encoders updated and isolate untrusted uploads at the operating-system level.

<a id="reading-optimization-reports"></a>

## Reading optimization reports

Use `optimize_with_report` or `optimize_to_with_report` with any chain, with or without safe output:

```rust,no_run
# fn main() -> Result<(), spatie_image_optimizer::Error> {
# let directory = tempfile::tempdir()?;
# let source = directory.path().join("image.jpg");
# std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/image.jpg"), &source)?;
# let path_to_image = source.as_path();
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
# Ok(())
# }
```

`input` and `output` identify the requested paths. `original_bytes` is the input size before optimization. `candidate_bytes` is the working size before acceptance, not the final output size; it can be larger or `None` after a failure. `bytes_saved()` counts published reduction against the source and returns zero for growth, retained originals and discarded candidates.

`optimizers` lists entries in execution order. `Skipped` means the optimizer does not handle that format; `Succeeded` does not guarantee smaller output. `Failed(String)` records a continued error. Total `elapsed` includes preparation and publication; per-tool times include support checks.

Without safe output, `Written` can include larger or partial output after a continued error. Use `on_error` or `throws(true)` for typed errors. Aborted chains return `Err`, not partial reports.
