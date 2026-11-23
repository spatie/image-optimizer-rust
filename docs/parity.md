---
title: PHP and Rust API
weight: 10
---

See [PARITY.md](https://github.com/spatie/image-optimizer-rust/blob/main/PARITY.md) for method mappings, configuration and the complete safety/reporting contract.

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

To preserve the source and use a separate destination:

```rust,ignore
use spatie_image_optimizer::OptimizerChainFactory;

let report = OptimizerChainFactory::create()
    .safe_output(true)
    .throws(true)
    .optimize_to_with_report(path_to_image, path_to_output)?;
println!("{:?}: {} bytes saved", report.status, report.bytes_saved());
```

Read the [safety contract](https://github.com/spatie/image-optimizer-rust/blob/main/PARITY.md#keeping-the-original-safe). This is not a sandbox or an image-validity check.

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

See the [field reference](https://github.com/spatie/image-optimizer-rust/blob/main/PARITY.md#reading-optimization-reports). Aborted chains return `Err`, not partial reports.
