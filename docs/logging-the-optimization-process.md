---
title: Logging the optimization process
weight: 6
---

By default the package will not throw any optimizer errors and just operate silently. To verify what the package is doing you can set a logger:

```rust,ignore
use spatie_image_optimizer::OptimizerChainFactory;

let optimizer_chain = OptimizerChainFactory::create();

optimizer_chain
    .use_logger(MyLogger)
    .optimize(path_to_image)?;
```

A logger is a type that implements `Logger`; `MyLogger` above is your application's logger. The package will write to log which optimizers are used, which commands are executed and their output. A closure receiving a `Level` and message also works.

If your application uses [tracing](https://docs.rs/tracing), pass `TracingLogger` to send these messages to its subscriber:

```rust,ignore
use spatie_image_optimizer::{OptimizerChainFactory, TracingLogger};

let optimizer_chain = OptimizerChainFactory::create();

optimizer_chain
    .use_logger(TracingLogger)
    .optimize(path_to_image)?;
```

Failures include their error details, whether the chain continues or returns an error. Validation and filesystem errors can still return `Err` with the default policy. Avoid confidential data in command arguments if you collect logs centrally.
