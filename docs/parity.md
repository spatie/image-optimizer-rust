---
title: PHP and Rust API
weight: 7
---

Reference: `spatie/image-optimizer` at [fc6cff0](https://github.com/spatie/image-optimizer/commit/fc6cff0c6f16c3a2616d909fa03c6dbc6062b39a).

| PHP feature | Rust API |
| --- | --- |
| `Image` construction, MIME, path, extension | `Image::new`, `mime`, `path`, `extension` |
| `OptimizerChainFactory::create()` | `OptimizerChainFactory::create()` |
| quality configuration | `OptimizerChainFactory::with_quality(q)?` |
| explicit factory optimizer config | `OptimizerChain::new().add_optimizer(...)` |
| chain inspection, append, replace | `get_optimizers`, `add_optimizer`, `set_optimizers` |
| `setTimeout` | `set_timeout(Duration)` |
| `useLogger` / `DummyLogger` | `use_logger` / `DummyLogger`, plus `TracingLogger` |
| `throws(true/false)` | `throws(true/false)` |
| `throws(callable)` | `on_error` returning `Result` |
| `optimize(path)` | `optimize(path)` |
| `optimize(input, output)` | `optimize_to(input, output)` |
| seven optimizer classes | `Tool::{Jpegoptim,Pngquant,Optipng,Svgo,Gifsicle,Cwebp,Avifenc}.with_options(...)` |
| options, binary path and executable overrides | `set_options`, `set_binary_path`, `set_binary_name`, `set_decode_binary_name` |
| image path + shell command construction | borrowed `Image` + `BinaryOptimizer::command` returning `Command` |
| `SelfHandlingOptimizer` and base class | `Optimizer` trait with `optimize` |
| `getTmpPath` and chain cleanup | RAII temporary PNG owned by AVIF optimization; custom optimizers own their resources |

All PHP optimization features, defaults and failure-policy modes are represented. Rust uses a single
trait for binary and self-handling implementations, typed tools instead of class-string configuration,
consuming builders and `Result` instead of mutation, mixed arrays and exceptions.

Deliberate differences: no shell evaluation of options, aliases rejected for source-preserving output,
MIME detection uses a bundled Rust database rather than PHP's system libmagic, only regular local files
are accepted, and Rust panics are not swallowed. File URLs resolve to local paths before execution.
Custom optimizers are `Send + Sync` and use RAII rather than a shared mutable temporary-path property.
Native-tool version differences can change encoded bytes and the set of accepted command options.

Rust-only additions are opt-in `.safe_output(true)` and the `optimize_with_report` /
`optimize_to_with_report` methods. Safe output preserves the destination on any optimizer failure
and only publishes smaller candidates; separate output falls back to an original-byte copy when
there is no reduction. Reports expose sizes, durations and per-optimizer outcomes without changing
the existing methods' return types. Default direct-write behavior is unchanged.
See [usage](usage.md#keeping-the-original-safe) for the full acceptance table and filesystem limits.
