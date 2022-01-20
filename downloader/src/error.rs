use reqwest::{StatusCode, Url};
use thiserror::Error;
use tokio::task::JoinError;

pub type Result<T> = core::result::Result<T, DownloaderError>;

#[derive(Error, Debug)]
pub enum DownloaderError {
    #[error("Input/Output error")]
    IoError(#[from] std::io::Error),
    #[error("Failed to perform a request")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Failed to parse URL")]
    UrlParseError(#[from] url::ParseError),
    #[error("Task join error")]
    JoinError(#[from] JoinError),
    #[error("Failed to download {0}, status code {1}")]
    DownloadError(Url, StatusCode),
}

impl<T> From<DownloaderError> for std::result::Result<T, DownloaderError> {
    fn from(error: DownloaderError) -> Self {
        Err(error)
    }
}
