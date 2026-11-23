---
title: PHP and Rust API
weight: 10
---

If you've used [spatie/image-optimizer](https://github.com/spatie/image-optimizer), most method names will look familiar. This table shows how to use the same operations in Rust.

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

Use the `Optimizer` trait for both command-line tools and optimizers that call an API. Configuration methods consume and return the chain, so keep the returned value or chain the calls together. Methods that can fail return `Result`; use `?` to pass an error to your caller.

## Differences to keep in mind

- Options are literal arguments, not shell commands. Pass a flag and its value as separate items, such as `["-q", "85"]`.
- Input must be a regular local file. Local file URLs are resolved to paths. A separate output must not point to the source, including through hard links.
- MIME detection uses a bundled Rust database rather than PHP's system libmagic, so unusual files may be classified differently.
- Rust panics are not caught. Custom optimizers must implement `Send + Sync` and own their temporary files so Rust can clean them up when they go out of scope.
- Native-tool versions affect encoded bytes and accepted options. Check the tool versions when comparing results across systems.

You can also enable [safe output](usage.md#keeping-the-original-safe) or request [optimization reports](usage.md#reading-optimization-reports). These are additions to the PHP API. They are opt-in; the default methods still write directly and return `Result<(), Error>`.
