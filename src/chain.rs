use crate::{DummyLogger, Error, Image, Level, Logger, Optimizer, Tool};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

type ErrorHandler = dyn Fn(&Error, &dyn Optimizer, &Image) -> Result<(), Error> + Send + Sync;

/// Whether the chain published its result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStatus {
    /// The candidate was written. Without safe output it may be larger or partial.
    Written,
    /// Safe output kept the original bytes because the candidate was not smaller.
    KeptOriginal,
    /// Safe output left the destination untouched because an optimizer failed.
    Discarded,
}

/// The result of one entry in the configured chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptimizerStatus {
    /// The optimizer returned successfully, which does not imply a size reduction.
    Succeeded,
    /// The optimizer did not support the current image.
    Skipped,
    /// The optimizer failed and the error policy allowed the chain to continue.
    Failed(String),
}

/// Timing and status of a configured optimizer, including skipped entries.
#[derive(Debug)]
pub struct OptimizerReport {
    /// Optimizer name, in configured execution order.
    pub name: String,
    /// Execution result or reason the optimizer did not run.
    pub status: OptimizerStatus,
    /// Time spent checking support and executing this optimizer.
    pub elapsed: Duration,
}

/// Result of a completed chain. Aborted chains return an [`Error`] instead.
#[derive(Debug)]
pub struct OptimizationReport {
    /// Requested input path, with file URLs resolved.
    pub input: PathBuf,
    /// Requested destination, not the safe-mode working path.
    pub output: PathBuf,
    /// Input size before running any optimizer.
    pub original_bytes: u64,
    /// Candidate size before acceptance, or `None` if a failed optimizer removed it.
    pub candidate_bytes: Option<u64>,
    /// Whether the destination was written, retained or left untouched on failure.
    pub status: OutputStatus,
    /// Total time, including copying, optimizer execution and publication.
    pub elapsed: Duration,
    /// Every configured optimizer, in execution order.
    pub optimizers: Vec<OptimizerReport>,
}

impl OptimizationReport {
    /// Bytes saved by a published candidate; zero for growth or discarded output.
    pub fn bytes_saved(&self) -> u64 {
        if self.status == OutputStatus::Written {
            self.original_bytes
                .saturating_sub(self.candidate_bytes.unwrap_or(self.original_bytes))
        } else {
            0
        }
    }
}

/// An ordered optimizer chain. By default failures are logged and optimization continues.
pub struct OptimizerChain {
    optimizers: Vec<Box<dyn Optimizer>>,
    logger: Box<dyn Logger>,
    timeout: Duration,
    throws: bool,
    error_handler: Option<Box<ErrorHandler>>,
    safe_output: bool,
}

impl std::fmt::Debug for OptimizerChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptimizerChain")
            .field("optimizer_count", &self.optimizers.len())
            .field("timeout", &self.timeout)
            .field("throws", &self.throws)
            .field("safe_output", &self.safe_output)
            .finish_non_exhaustive()
    }
}

impl Default for OptimizerChain {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizerChain {
    /// Create an empty chain with a 60-second timeout per optimizer.
    pub fn new() -> Self {
        Self {
            optimizers: Vec::new(),
            logger: Box::new(DummyLogger),
            timeout: Duration::from_secs(60),
            throws: false,
            error_handler: None,
            safe_output: false,
        }
    }

    /// Inspect the optimizers in execution order.
    pub fn get_optimizers(&self) -> &[Box<dyn Optimizer>] {
        &self.optimizers
    }

    /// Append a binary or self-handling optimizer.
    pub fn add_optimizer(mut self, optimizer: impl Optimizer + 'static) -> Self {
        self.optimizers.push(Box::new(optimizer));
        self
    }

    /// Replace the entire chain, retaining the supplied order.
    pub fn set_optimizers(mut self, optimizers: Vec<Box<dyn Optimizer>>) -> Self {
        self.optimizers = optimizers;
        self
    }

    /// Set the time budget per optimizer. Zero disables binary timeouts.
    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Work on a private copy and publish only smaller, error-free output.
    ///
    /// Disabled by default. Safe output requires a writable destination directory,
    /// rejects destination symlinks and replaces files by rename. See the usage guide
    /// for permissions, hard links and durability limits.
    pub fn safe_output(mut self, enabled: bool) -> Self {
        self.safe_output = enabled;
        self
    }

    /// Replace the default silent logger.
    pub fn use_logger(mut self, logger: impl Logger + 'static) -> Self {
        self.logger = Box::new(logger);
        self
    }

    /// Enable immediate failure or restore log-and-continue. Replaces a custom handler.
    pub fn throws(mut self, enabled: bool) -> Self {
        self.throws = enabled;
        self.error_handler = None;
        self
    }

    /// Handle optimizer errors. Return `Ok(())` to continue or `Err` to abort.
    ///
    /// Validation and output-copy failures are always returned directly.
    pub fn on_error<F>(mut self, handler: F) -> Self
    where
        F: Fn(&Error, &dyn Optimizer, &Image) -> Result<(), Error> + Send + Sync + 'static,
    {
        self.error_handler = Some(Box::new(handler));
        self
    }

    /// Optimize a local file in place.
    pub fn optimize(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        self.optimize_with_report(path).map(|_| ())
    }

    /// Optimize in place and report sizes, timing and every configured optimizer.
    pub fn optimize_with_report(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<OptimizationReport, Error> {
        let input = Image::new(path)?;
        let output = input.path().to_owned();
        self.run(input, output, false)
    }

    /// Copy to an output file, then optimize the copy, preserving the source.
    ///
    /// Parent directories must exist. Both paths must be local files.
    pub fn optimize_to(
        &self,
        input: impl AsRef<Path>,
        output: impl AsRef<Path>,
    ) -> Result<(), Error> {
        self.optimize_to_with_report(input, output).map(|_| ())
    }

    /// Optimize to a separate path and report the candidate and publication status.
    ///
    /// With safe output, a non-smaller candidate produces an original-byte copy.
    /// Any optimizer failure leaves the destination untouched, even in continue mode.
    pub fn optimize_to_with_report(
        &self,
        input: impl AsRef<Path>,
        output: impl AsRef<Path>,
    ) -> Result<OptimizationReport, Error> {
        let input = Image::new(input)?;
        let output = crate::image::local_path(output.as_ref())?;
        // Reject aliases before copy, which otherwise truncates the input on some platforms.
        if output.exists() && same_file::is_same_file(input.path(), &output)? {
            return Err(Error::InvalidPath(
                "Input and output must be different files.".into(),
            ));
        }
        self.run(input, output, true)
    }

    fn run(
        &self,
        input: Image,
        output: PathBuf,
        separate_output: bool,
    ) -> Result<OptimizationReport, Error> {
        let started = Instant::now();
        let original_bytes = fs::metadata(input.path())?.len();
        let temporary = if self.safe_output {
            if fs::symlink_metadata(&output).is_ok_and(|m| m.file_type().is_symlink()) {
                return Err(Error::InvalidPath(
                    "Safe output rejects destination symlinks.".into(),
                ));
            }
            let parent = output
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let mut builder = tempfile::Builder::new();
            builder.prefix(".spatie-optimizer-");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                builder.permissions(fs::Permissions::from_mode(0o700));
            }
            Some(builder.tempdir_in(parent)?)
        } else {
            None
        };
        let working = if let Some(temporary) = &temporary {
            temporary.path().join(
                output
                    .file_name()
                    .ok_or_else(|| Error::InvalidPath(output.display().to_string()))?,
            )
        } else {
            output.clone()
        };
        if separate_output || self.safe_output {
            fs::copy(input.path(), &working).map_err(|source| Error::Copy {
                path: output.clone(),
                source,
            })?;
        }
        let optimizers = self.apply(Image::new(&working)?)?;
        let failed = optimizers
            .iter()
            .any(|r| matches!(r.status, OptimizerStatus::Failed(_)));
        let candidate_bytes = if failed {
            fs::metadata(&working).ok().map(|m| m.len())
        } else {
            Some(fs::metadata(&working)?.len())
        };
        let status = if self.safe_output && failed {
            OutputStatus::Discarded
        } else if self.safe_output && candidate_bytes.is_some_and(|size| size >= original_bytes) {
            if separate_output {
                fs::copy(input.path(), &working)?;
            }
            OutputStatus::KeptOriginal
        } else {
            OutputStatus::Written
        };
        if self.safe_output
            && (status == OutputStatus::Written
                || (separate_output && status == OutputStatus::KeptOriginal))
        {
            let permissions = match fs::metadata(&output) {
                Ok(metadata) => metadata.permissions(),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    fs::metadata(input.path())?.permissions()
                }
                Err(error) => return Err(error.into()),
            };
            fs::set_permissions(&working, permissions)?;
            fs::rename(&working, &output)?;
        }
        Ok(OptimizationReport {
            input: input.path().to_owned(),
            output,
            original_bytes,
            candidate_bytes,
            status,
            elapsed: started.elapsed(),
            optimizers,
        })
    }

    fn apply(&self, image: Image) -> Result<Vec<OptimizerReport>, Error> {
        let mut reports = Vec::new();
        self.logger.log(
            Level::Info,
            &format!("Start optimizing {}", image.path().display()),
        );
        for optimizer in &self.optimizers {
            let started = Instant::now();
            if !optimizer.can_handle(&image) {
                reports.push(OptimizerReport {
                    name: optimizer.name().into(),
                    status: OptimizerStatus::Skipped,
                    elapsed: started.elapsed(),
                });
                continue;
            }
            self.logger.log(
                Level::Info,
                &format!("Using optimizer: `{}`", optimizer.name()),
            );
            let status = if let Err(error) = optimizer.optimize(&image, &*self.logger, self.timeout)
            {
                self.logger
                    .log(Level::Error, &format!("Optimizer errored with `{error}`"));
                if let Some(handler) = &self.error_handler {
                    handler(&error, &**optimizer, &image)?;
                } else if self.throws {
                    return Err(error);
                }
                OptimizerStatus::Failed(error.to_string())
            } else {
                OptimizerStatus::Succeeded
            };
            reports.push(OptimizerReport {
                name: optimizer.name().into(),
                status,
                elapsed: started.elapsed(),
            });
        }
        Ok(reports)
    }
}

/// Build the PHP package's default chain and quality settings.
#[derive(Debug)]
pub struct OptimizerChainFactory;

impl OptimizerChainFactory {
    /// Create the default seven-optimizer chain.
    pub fn create() -> OptimizerChain {
        Self::build(None)
    }

    /// Apply a quality from 0 through 100 to JPEG, PNG, WebP and AVIF encoders.
    pub fn with_quality(quality: u8) -> Result<OptimizerChain, Error> {
        if quality > 100 {
            return Err(Error::InvalidQuality(quality));
        }
        Ok(Self::build(Some(quality)))
    }

    fn build(quality: Option<u8>) -> OptimizerChain {
        let jpeg = quality.unwrap_or(85);
        let webp = quality.unwrap_or(90);
        let avif = quality.map_or(23, |q| (63.0 - f64::from(q) * 0.63).round() as u8);
        OptimizerChain::new()
            .add_optimizer(Tool::Jpegoptim.with_options([
                format!("-m{jpeg}"),
                "--force".into(),
                "--strip-all".into(),
                "--all-progressive".into(),
            ]))
            .add_optimizer(
                Tool::Pngquant.with_options([format!("--quality={jpeg}"), "--force".into()]),
            )
            .add_optimizer(Tool::Optipng.with_options(["-i0", "-o2", "-quiet"]))
            .add_optimizer(Tool::Svgo.with_options(Vec::<String>::new()))
            .add_optimizer(Tool::Gifsicle.with_options(["-b", "-O3"]))
            .add_optimizer(Tool::Cwebp.with_options([
                "-m",
                "6",
                "-pass",
                "10",
                "-mt",
                "-q",
                &webp.to_string(),
            ]))
            .add_optimizer(Tool::Avifenc.with_options([
                "-a",
                &format!("cq-level={avif}"),
                "-j",
                "all",
                "--min",
                "0",
                "--max",
                "63",
                "--minalpha",
                "0",
                "--maxalpha",
                "63",
                "-a",
                "end-usage=q",
                "-a",
                "tune=ssim",
            ]))
    }
}
