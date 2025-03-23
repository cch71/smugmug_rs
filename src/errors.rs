use std::io;
use thiserror::Error;

/// Error conditions that can be returned
#[derive(Error, Debug)]
pub enum SmugMugError {
    #[error("I/O error")]
    Io(#[from] io::Error),

    #[error("Request network error")]
    Request(#[from] reqwest::Error),

    #[error("Authorization error")]
    Auth(#[from] reqwest_oauth1::Error),

    #[error("Deserialization error")]
    Deserialization(#[from] serde_json::Error),

    #[error("URL Parse error")]
    UrlParsing(#[from] url::ParseError),

    // #[error("the data for key `{0}` is not available")]
    // Redaction(String),
    // #[error("invalid header (expected {expected:?}, found {found:?})")]
    // InvalidHeader {
    //     expected: String,
    //     found: String,
    // },
    #[error("unknown data store error")]
    Unknown,
}

