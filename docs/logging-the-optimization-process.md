---
title: Logging the optimization process
weight: 6
---

By default the package operates silently. To verify what the package is doing, you can set a logger:

```rust,no_run
use spatie_image_optimizer::{Level, OptimizerChainFactory};

let optimizer_chain = OptimizerChainFactory::create();
optimizer_chain
    .use_logger(|level: Level, message: &str| eprintln!("{level:?}: {message}"))
    .optimize("photo.jpg")?;
# Ok::<(), spatie_image_optimizer::Error>(())
```

A logger implements the `Logger` trait. A closure receiving a `Level` and message also works, as shown above. The package logs which optimizers are used, which commands are executed and their output. Failures include their error details, whether the chain continues or returns an error.

If your application uses [tracing](https://docs.rs/tracing), pass `TracingLogger` to send these messages to its subscriber:

```rust,no_run
use spatie_image_optimizer::{OptimizerChainFactory, TracingLogger};

OptimizerChainFactory::create()
    .use_logger(TracingLogger)
    .optimize("photo.jpg")?;
# Ok::<(), spatie_image_optimizer::Error>(())
```

Avoid confidential data in command arguments if you collect logs centrally.
