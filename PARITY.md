# PHP and Rust API reference

This reference records the differences from [spatie/image-optimizer](https://github.com/spatie/image-optimizer). The [usage guide](docs/usage.md) follows the PHP README; the details below cover Rust configuration, safety and reporting.

## Corresponding methods

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

Configuration methods consume and return the chain, so keep the returned value or chain the calls together. Methods that can fail return `Result`; use `?` to pass an error to your caller. Rust panics are not caught. Custom optimizers must implement `Send + Sync` and own their temporary files so Rust can clean them up when they go out of scope.

## Inputs and native tools

Input must be a regular local file. Local file URLs are resolved to paths. MIME detection uses a bundled Rust database rather than PHP's system libmagic, so unusual files may be classified differently. Non-image files pass through unchanged.

Missing binaries are logged failures, not skipped by an installation check. Native-tool versions affect encoded bytes and accepted options. Check the tool versions when comparing results across systems. Node is required only for the external SVGO tool, not for the Rust library. On Windows install native executables and add them to `PATH`, or use `set_binary_path`.

The Rust defaults differ from PHP in these places:

- Jpegoptim also receives `--force`, which allows larger output. Enable safe output to reject growth.
- Pngquant receives `--quality=85` and `--force` rather than using its defaults. Optipng also receives `-quiet`.
- SVGO uses its installed default configuration; no plugins are disabled. To preserve IDs or a `viewBox`, pass `--config` and the path to your SVGO configuration using `with_options` on `Tool::Svgo`. Check the rendered image, especially when other elements reference its IDs.
- Gifsicle also receives `-b` to overwrite the input.
- AVIF needs both `avifdec` and `avifenc` (libavif 0.9.3 or newer with the documented options). Before encoding, `avifdec -j all --ignore-icc --no-strict --png-compress 0` decodes the image into a temporary PNG. Both commands share the optimizer's timeout budget. The PNG is removed on success or error. Jobs can use many CPU threads by default; supply `["-j", "1"]` in custom encoder options when resources are limited.

`OptimizerChainFactory::with_quality(q)` sets JPEG and PNG quality to `q`, WebP quality to `q`, and AVIF `cq-level` to `round(63 - q * 0.63)`. Valid quality is 0 through 100. Without an explicit quality, AVIF uses 23 and WebP uses 90.

## Chain configuration and errors

The default timeout is 60 seconds per optimizer, not per chain. Zero disables binary timeouts. A timed-out binary is killed and reaped, but independently spawned descendants are not. Custom self-handling optimizers must enforce their own timeout. Keep scratch files in owners such as `tempfile::NamedTempFile` so they are removed before the error callback runs.

Calling `throws` replaces a custom handler. A process timeout returns `Error::Timeout`. `Error::Process` includes the exit code, stdout and stderr. Validation and output-copy errors always return `Err`; Rust panics are not caught.

Options are literal arguments. Use `["-q", "85"]`, not `["-q 85"]`; no shell expansion, pipelines or redirection occurs. Use `set_options` to replace arguments, `set_binary_path` to select a directory containing the binaries, `set_binary_name` to select another executable, and `set_decode_binary_name` to override the AVIF decoder. `set_optimizers(Vec<Box<dyn Optimizer>>)` replaces the chain; `get_optimizers()` gives you its entries in execution order. Binary optimizers can use `run_command` to run a `Command` with logging and a timeout.

Use the `Optimizer` trait for both command-line tools and optimizers that call an API. The chain delegates execution to its `optimize` method. Set your network client's timeout from the supplied duration.

## Logging

A logger implements `Logger`. A closure receiving a `Level` and message also works. `TracingLogger` sends messages to your application's [tracing](https://docs.rs/tracing) subscriber. Failures include their error details, whether the chain continues or returns an error. Validation and filesystem errors can still return `Err` with the default policy. Avoid confidential data in command arguments if you collect logs centrally.

## Keeping the original safe

Safe output and optimization reports are opt-in additions to the PHP API. The default methods still write directly and return `Result<(), Error>`.

Neither `optimize` nor `optimize_to` guarantees a smaller image in the default direct-write mode. A failed optimizer can leave partial changes. For separate output, the output directory must already exist, and the output must not refer to the source, including through a hard link.

Enable `safe_output(true)` to optimize a working copy before replacing the destination. The [safe-output examples](docs/safe-output-and-reports.md#keeping-the-original-safe) show in-place and separate-output calls.

The chain works on a temporary copy in the destination's parent directory, preserving the filename and extension for tools such as SVGO. Use a trusted, writable parent with enough space for that copy and optimizer scratch files. The temporary directory uses mode `0o700` on Unix and inherits the parent's ACL on Windows.

| Result | In-place optimization | Separate output |
| --- | --- | --- |
| All matching tools succeed and the candidate is smaller | Replace the original | Write the smaller candidate |
| Candidate is equal in size or larger, or no tools match | Leave the original untouched | Write a byte-for-byte copy of the source |
| Any optimizer fails and the error policy continues | Leave the original untouched | Leave an existing destination untouched; do not create a new one |
| Error policy aborts, or copying/publication fails | Return `Err` | Return `Err` |

Safe output does not change `throws` or `on_error`. Continued errors let later tools run but discard the whole candidate, even if a later tool succeeds. Missing binaries and timeouts are failures, not skips. Check for `Discarded`: `Ok` alone does not mean an optimized file was published. Use `.throws(true)` to return `Err` on any optimizer failure.

Publication uses a same-filesystem rename, atomic where the filesystem supports it. There is no `fsync` or power-loss durability guarantee. Normal return and error unwinding remove the working directory; forced termination may leave it behind.

Destination symlinks, including broken ones, and separate-output source aliases are rejected. Replacement creates a new file; other hard links and open handles still refer to the old one. Existing destinations retain their `std::fs::Permissions`; new outputs copy the source's permissions. Ownership, ACLs, extended attributes and timestamps may change. Read-only destinations can prevent replacement, especially on Windows.

Custom optimizers and callbacks receive the working path; reports identify the requested paths. Configuration discovery relative to the image may require an explicit configuration path. Safe output trusts encoders: it does not validate pixels, automatically preserve metadata, sandbox programs or prevent concurrent path changes.

`optimize` and `optimize_to` also honor safe output but discard the report. Call `.safe_output(false)` to restore direct writes. Native image tools process untrusted data in your process's security context; keep them updated and use operating-system isolation for untrusted uploads.

## Reading optimization reports

Use `optimize_with_report` or `optimize_to_with_report` with any chain, with or without safe output. The [report example](docs/safe-output-and-reports.md#reading-optimization-reports) prints sizes, timing and per-optimizer outcomes.

- `input` and `output` are requested local paths. File URLs are resolved, not canonicalized.
- `original_bytes` is the input size before optimization.
- `candidate_bytes` is the working size **before** accepting or discarding it, not necessarily the final destination size. It may be larger, or `None` if failure left no readable candidate.
- `bytes_saved()` counts published reduction against the source, not a pre-existing output. Growth, retained originals and missing or discarded candidates return zero.
- `optimizers` lists all entries in order. `Skipped` means `can_handle` returned false; `Succeeded` means `Ok`, not necessarily smaller output. `Failed(String)` holds the continued error's display text, not a typed error.
- `elapsed` includes preparation, execution and publication; per-tool times include support checks.

Without safe output, `Written` can mean larger, invalid or partial output after a continued error. Use `on_error` or `.throws(true)` for typed process errors. Aborted chains return `Err`, not partial reports.
