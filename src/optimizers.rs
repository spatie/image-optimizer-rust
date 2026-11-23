use crate::{Error, Image, Logger, Optimizer, run_command};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// The seven native optimizers provided by the PHP package, in default chain order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tool {
    /// JPEG compression with `jpegoptim`.
    Jpegoptim,
    /// Lossy PNG compression with `pngquant`.
    Pngquant,
    /// Lossless PNG compression with `optipng`.
    Optipng,
    /// SVG optimization with `svgo`.
    Svgo,
    /// GIF optimization with `gifsicle`.
    Gifsicle,
    /// WebP compression with `cwebp`.
    Cwebp,
    /// AVIF decoding with `avifdec`, then encoding with `avifenc`.
    Avifenc,
}

impl Tool {
    /// Build this optimizer with explicit command arguments.
    ///
    /// Each item is one argument, e.g. `["-q", "85"]`, not `["-q 85"]`.
    pub fn with_options<I, S>(self, options: I) -> BinaryOptimizer
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        BinaryOptimizer {
            tool: self,
            options: options.into_iter().map(Into::into).collect(),
            binary_path: PathBuf::new(),
            binary_name: self.binary_name().into(),
            decode_binary_name: "avifdec".into(),
        }
    }

    /// Return the default executable name.
    pub fn binary_name(self) -> &'static str {
        match self {
            Self::Jpegoptim => "jpegoptim",
            Self::Pngquant => "pngquant",
            Self::Optipng => "optipng",
            Self::Svgo => "svgo",
            Self::Gifsicle => "gifsicle",
            Self::Cwebp => "cwebp",
            Self::Avifenc => "avifenc",
        }
    }
}

/// A native optimizer with configurable executable names, location and arguments.
#[derive(Clone, Debug)]
pub struct BinaryOptimizer {
    tool: Tool,
    options: Vec<OsString>,
    binary_path: PathBuf,
    binary_name: OsString,
    decode_binary_name: OsString,
}

impl BinaryOptimizer {
    /// Return the optimizer kind.
    pub fn tool(&self) -> Tool {
        self.tool
    }

    /// Return the configured encoder executable name.
    pub fn binary_name(&self) -> &OsStr {
        &self.binary_name
    }

    /// Return configured arguments.
    pub fn options(&self) -> &[OsString] {
        &self.options
    }

    /// Replace the command arguments. Each item is passed as a single argument.
    pub fn set_options<I, S>(mut self, options: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.options = options.into_iter().map(Into::into).collect();
        self
    }

    /// Set a directory containing the executables. An empty path uses `PATH`.
    pub fn set_binary_path(mut self, path: impl AsRef<Path>) -> Self {
        self.binary_path = path.as_ref().into();
        self
    }

    /// Override the encoder executable name or path.
    pub fn set_binary_name(mut self, name: impl Into<OsString>) -> Self {
        self.binary_name = name.into();
        self
    }

    /// Override the AVIF decoder executable name or path.
    pub fn set_decode_binary_name(mut self, name: impl Into<OsString>) -> Self {
        self.decode_binary_name = name.into();
        self
    }

    /// Build the encoder command for inspection or custom execution.
    ///
    /// For AVIF, `input` must be a decoded PNG. The standard optimizer reserves
    /// that PNG and executes the decoder first.
    pub fn command(&self, input: &Path, output: &Path) -> Command {
        let mut command = Command::new(self.binary_path.join(&self.binary_name));
        command.args(&self.options);
        match self.tool {
            Tool::Svgo => {
                command
                    .arg("--input")
                    .arg(input)
                    .arg("--output")
                    .arg(output);
            }
            Tool::Pngquant => {
                command.arg(input).arg("--output").arg(output);
            }
            Tool::Gifsicle => {
                command.arg(input).arg("--output").arg(output);
            }
            Tool::Cwebp => {
                command.arg(input).arg("-o").arg(output);
            }
            Tool::Avifenc => {
                command.arg(input).arg(output);
            }
            Tool::Jpegoptim | Tool::Optipng => {
                command.arg(input);
            }
        }
        command
    }
}

impl Optimizer for BinaryOptimizer {
    fn name(&self) -> &str {
        self.tool.binary_name()
    }

    fn can_handle(&self, image: &Image) -> bool {
        let Ok(mime) = image.mime() else {
            return false;
        };
        match self.tool {
            Tool::Jpegoptim => mime == "image/jpeg",
            Tool::Pngquant | Tool::Optipng => mime == "image/png",
            Tool::Svgo => {
                image.extension() == "svg"
                    && matches!(
                        mime.as_str(),
                        "text/html" | "image/svg" | "image/svg+xml" | "text/plain"
                    )
            }
            Tool::Gifsicle => mime == "image/gif",
            Tool::Cwebp => mime == "image/webp",
            Tool::Avifenc => mime == "image/avif",
        }
    }

    fn optimize(&self, image: &Image, logger: &dyn Logger, timeout: Duration) -> Result<(), Error> {
        let path = std::path::absolute(image.path())?;
        if self.tool != Tool::Avifenc {
            return run_command(&mut self.command(&path, &path), timeout, logger);
        }
        let start = Instant::now();
        let decoded = tempfile::Builder::new()
            .prefix("avifdec-")
            .suffix(".png")
            .tempfile()?;
        let mut decode = Command::new(self.binary_path.join(&self.decode_binary_name));
        decode
            .args([
                "-j",
                "all",
                "--ignore-icc",
                "--no-strict",
                "--png-compress",
                "0",
            ])
            .arg(&path)
            .arg(decoded.path());
        run_command(&mut decode, timeout, logger)?;
        let remaining = if timeout.is_zero() {
            timeout
        } else {
            timeout
                .checked_sub(start.elapsed())
                .filter(|time| !time.is_zero())
                .ok_or_else(|| Error::Timeout {
                    program: self.binary_path.join(&self.binary_name),
                    timeout,
                })?
        };
        run_command(&mut self.command(decoded.path(), &path), remaining, logger)
    }
}
