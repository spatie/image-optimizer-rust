#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod chain;
mod image;
mod optimizers;
mod process;

pub use chain::{
    OptimizationReport, OptimizerChain, OptimizerChainFactory, OptimizerReport, OptimizerStatus,
    OutputStatus,
};
pub use image::Image;
pub use optimizers::{BinaryOptimizer, Tool};
pub use process::run_command;

use std::path::PathBuf;
use std::time::Duration;

/// Configuration, filesystem, optimizer or process failure.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Only local paths and local file URLs are supported.
    #[error("Only local files are supported: {0}")]
    InvalidPath(String),
    /// A requested quality is outside 0 through 100.
    #[error("Quality must be between 0 and 100, got {0}.")]
    InvalidQuality(u8),
    /// Copying the input to the requested output failed before optimization.
    #[error("Cannot copy file to `{path}`: {source}")]
    Copy {
        /// Requested output path.
        path: PathBuf,
        /// Operating system error.
        #[source]
        source: std::io::Error,
    },
    /// A process exited unsuccessfully, with captured output for diagnostics.
    #[error("Optimizer `{program}` exited with status {status:?}: {stderr}")]
    Process {
        /// Executed binary.
        program: PathBuf,
        /// Exit code, or `None` when terminated by a signal.
        status: Option<i32>,
        /// Standard output.
        stdout: String,
        /// Standard error.
        stderr: String,
    },
    /// An optimizer exceeded its time budget and was killed and reaped.
    #[error("Optimizer `{program}` exceeded {timeout:?}.")]
    Timeout {
        /// Executed binary.
        program: PathBuf,
        /// Configured time budget.
        timeout: Duration,
    },
    /// An application-defined optimizer or error handler failed.
    #[error("{0}")]
    Custom(String),
    /// An operating system operation failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Log severity used by the chain and supplied optimizers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Level {
    /// Progress, commands and successful process output.
    Info,
    /// Optimizer failures, whether continued or returned.
    Error,
}

/// Receives chain progress and optimizer diagnostics.
pub trait Logger: Send + Sync {
    /// Record a message.
    fn log(&self, level: Level, message: &str);
}

impl<F: Fn(Level, &str) + Send + Sync> Logger for F {
    fn log(&self, level: Level, message: &str) {
        self(level, message);
    }
}

/// Default silent logger.
#[derive(Debug)]
pub struct DummyLogger;

impl Logger for DummyLogger {
    fn log(&self, _: Level, _: &str) {}
}

/// Routes log messages to the application's `tracing` subscriber.
#[derive(Debug)]
pub struct TracingLogger;

impl Logger for TracingLogger {
    fn log(&self, level: Level, message: &str) {
        match level {
            Level::Info => tracing::info!("{message}"),
            Level::Error => tracing::error!("{message}"),
        }
    }
}

/// An optimizer backed by a binary, a service, or application code.
///
/// Return errors instead of panicking. Self-handling implementations enforce
/// their own timeouts; binary implementations can use [`run_command`]. Own any
/// scratch files with RAII so they are removed before returning, including errors.
pub trait Optimizer: Send + Sync {
    /// Name used in logs and error callbacks.
    fn name(&self) -> &str;
    /// Whether this optimizer supports the image.
    fn can_handle(&self, image: &Image) -> bool;
    /// Optimize the image in place. A zero timeout means no deadline.
    fn optimize(&self, image: &Image, logger: &dyn Logger, timeout: Duration) -> Result<(), Error>;
}

#[cfg(doctest)]
mod guide {
    #[doc = include_str!("../docs/introduction.md")]
    mod introduction {}
    #[doc = include_str!("../docs/usage.md")]
    mod usage {}
    #[doc = include_str!("../docs/logging-the-optimization-process.md")]
    mod logging {}
    #[doc = include_str!("../docs/parity.md")]
    mod parity {}
}
