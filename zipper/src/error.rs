use std::path::PathBuf;
use thiserror::Error;
use zip::result::ZipError;

pub type Result<T> = core::result::Result<T, ZipperError>;

#[derive(Error, Debug)]
pub enum ZipperError {
    #[error("Input/Output error")]
    IoError(#[from] std::io::Error),
    #[error("Zip error")]
    ZipError(#[from] ZipError),
    #[error("Walkdir error")]
    WalkdirError(#[from] walkdir::Error),
    #[error("Unknown entry type {0}")]
    UnknownEntryType(PathBuf),
    #[cfg(feature = "file-matcher")]
    #[error("File matcher error")]
    FileMatcherError(#[from] file_matcher::FileMatcherError),
}

impl<T> From<ZipperError> for std::result::Result<T, ZipperError> {
    fn from(error: ZipperError) -> Self {
        Err(error)
    }
}
