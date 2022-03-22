use thiserror::Error;
use zip::result::ZipError;

pub type Result<T> = core::result::Result<T, UnzipperError>;

#[derive(Error, Debug)]
pub enum UnzipperError {
    #[error("Input/Output error")]
    IoError(#[from] std::io::Error),
    #[error("Zip error")]
    ZipError(#[from] ZipError),
    #[error("Walkdir error")]
    WalkdirError(#[from] walkdir::Error),
}

impl<T> From<UnzipperError> for std::result::Result<T, UnzipperError> {
    fn from(error: UnzipperError) -> Self {
        Err(error)
    }
}
