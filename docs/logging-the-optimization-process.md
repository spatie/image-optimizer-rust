---
title: Logging the optimization process
weight: 6
---

By default the package will not throw any optimizer errors and just operate silently. To verify what the package is doing you can set a logger:

```rust,no_run
# fn main() -> Result<(), spatie_image_optimizer::Error> {
# let directory = tempfile::tempdir()?;
# let source = directory.path().join("image.jpg");
# std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/image.jpg"), &source)?;
# let path_to_image = source.as_path();
# struct MyLogger;
# impl spatie_image_optimizer::Logger for MyLogger {
#     fn log(&self, level: spatie_image_optimizer::Level, message: &str) { eprintln!("{level:?}: {message}"); }
# }
use spatie_image_optimizer::OptimizerChainFactory;

let optimizer_chain = OptimizerChainFactory::create();

optimizer_chain
    .use_logger(MyLogger)
    .optimize(path_to_image)?;
# Ok(())
# }
```

A logger is a type that implements `Logger`. The package will write to log which optimizers are used, which commands are executed and their output. `MyLogger` above is your application's logger.

If your application uses [tracing](https://docs.rs/tracing), pass `TracingLogger` to send these messages to its subscriber:

```rust,no_run
# fn main() -> Result<(), spatie_image_optimizer::Error> {
# let directory = tempfile::tempdir()?;
# let source = directory.path().join("image.jpg");
# std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/image.jpg"), &source)?;
# let path_to_image = source.as_path();
use spatie_image_optimizer::{OptimizerChainFactory, TracingLogger};

let optimizer_chain = OptimizerChainFactory::create();

optimizer_chain
    .use_logger(TracingLogger)
    .optimize(path_to_image)?;
# Ok(())
# }
```

A closure receiving a `Level` and message also implements `Logger`. Failures include their error details whether the chain continues or aborts. Avoid confidential command arguments when collecting logs centrally. Validation and filesystem failures still return `Err` with the default policy.
