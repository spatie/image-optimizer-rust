# Changelog

## 1.0.1 - 2026-11-24

- Give safe output and optimization reports a dedicated guide, including filesystem limits.

## 1.0.0 - 2026-11-24

- Compile all guide examples with rustdoc using hidden setup, and execute the same examples in native-tool tests.
- Add opt-in safe output, size-based acceptance and atomic replacement where supported by the filesystem.
- Add optimization reports with candidate size, published savings, timings and per-optimizer outcomes.
- Document error handling, unchanged output, destination preservation and filesystem limits.
- Port the seven native optimizers, chain configuration, quality defaults, logging, timeouts and error policies.
- Add binary and self-handling custom optimizers through one trait and RAII cleanup for AVIF intermediates.
- Add standalone Cargo tooling, guide pages and portable and native-tool integration suites.
