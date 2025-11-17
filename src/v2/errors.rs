/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */

use crate::v2::ApiErrorCodes;
use num_enum::TryFromPrimitiveError;
use std::io;
use thiserror::Error;

/// Error conditions that can be returned
#[derive(Error, Debug)]
pub enum SmugMugError {
    #[error("I/O error")]
    Io(#[from] io::Error),

    #[error("Request network error")]
    Request(#[from] reqwest::Error),

    #[error("Authorization error. {0}")]
    Auth(String),

    #[error("Deserialization error")]
    Deserialization(#[from] serde_json::Error),

    #[error("URL Parse error")]
    UrlParsing(#[from] url::ParseError),

    #[error("This is not an album")]
    NotAnAlbum(),

    #[error("Client not found")]
    ClientNotFound(),

    #[error("Image archive not found for: {0} image key:{1}")]
    ImageArchiveNotFound(String, String),

    #[error("Expected response missing")]
    ResponseMissing(),

    #[error("API Response was error: {0}, msg: {1}")]
    ApiResponse(u32, String),

    #[error("API Response error code is invalid")]
    ApiResponseCode(#[from] TryFromPrimitiveError<ApiErrorCodes>),

    #[error("API Response is a too many requests error. Retry after {0} seconds")]
    ApiResponseTooManyRequests(u64),

    #[error("API Response is malformed: {0:?}")]
    ApiResponseMalformed(serde_json::Error),

    #[error("Failed serializing to JSON: {0}")]
    JsonSerialization(String),
}
