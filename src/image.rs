use crate::Error;
use std::path::{Path, PathBuf};

/// A validated local image path. Unsupported file types can still pass through a chain.
#[derive(Debug, Clone)]
pub struct Image {
    path: PathBuf,
}

impl Image {
    /// Validate a local file path or `file://` URL. Network URLs are rejected.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = local_path(path.as_ref())?;
        if !path.is_file() {
            return Err(Error::InvalidPath(path.display().to_string()));
        }
        Ok(Self { path })
    }

    /// Detect MIME type from the current file contents, not the extension.
    pub fn mime(&self) -> Result<String, Error> {
        Ok(file_format::FileFormat::from_file(&self.path)?
            .media_type()
            .to_owned())
    }

    /// Return the local filesystem path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Return a lowercase extension without a leading dot.
    pub fn extension(&self) -> String {
        self.path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase()
    }
}

pub(crate) fn local_path(path: &Path) -> Result<PathBuf, Error> {
    if let Some(text) = path.to_str() {
        if text
            .get(..5)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("file:"))
        {
            return url::Url::parse(text)
                .ok()
                .and_then(|url| url.to_file_path().ok())
                .ok_or_else(|| Error::InvalidPath(text.into()));
        }
        if let Some((scheme, _)) = text.split_once(':') {
            let windows_drive =
                cfg!(windows) && scheme.len() == 1 && scheme.as_bytes()[0].is_ascii_alphabetic();
            if !windows_drive && !scheme.contains(['/', '\\']) {
                return Err(Error::InvalidPath(text.into()));
            }
        }
    }
    Ok(path.into())
}
