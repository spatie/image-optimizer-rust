---
title: Logging the optimization process
weight: 5
---

The default `DummyLogger` is silent. Use `TracingLogger` with your application's `tracing` subscriber,
or pass a closure that receives a `Level` and message:

```rust,no_run
use spatie_image_optimizer::{Level, OptimizerChainFactory, TracingLogger};

let chain = OptimizerChainFactory::create().use_logger(TracingLogger);
let chain = chain.use_logger(|level: Level, message: &str| eprintln!("{level:?}: {message}"));
chain.optimize("photo.jpg")?;
# Ok::<(), spatie_image_optimizer::Error>(())
```

Logs include the image path, selected optimizers, executed command arguments and successful stdout.
Failures include their error details. Error policy does not change logging. Avoid confidential data
in command arguments if you collect logs centrally.
