/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::errors::SmugMugError;
use crate::v2::{API_ORIGIN, Client};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;

/// Holds information returned from the AlbumImage/Image API.
///
/// See [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/reference/image.html) for more
/// details on the individual fields.
#[derive(Deserialize, Debug)]
pub struct Image {
    #[serde(skip)]
    pub(crate) client: Arc<Client>,

    #[serde(rename = "Uri")]
    pub uri: String,

    #[serde(rename = "Title")]
    pub name: String,

    #[serde(rename = "Caption")]
    pub description: String,

    #[serde(rename = "Altitude")]
    pub altitude: u64,

    #[serde(rename = "Latitude")]
    pub latitude: String,

    #[serde(rename = "Longitude")]
    pub longitude: String,

    #[serde(rename = "Format")]
    pub format: String,

    #[serde(rename = "FileName")]
    pub file_name: String,

    #[serde(rename = "KeywordArray")]
    pub keywords: Vec<String>,

    #[serde(rename = "ArchivedUri")]
    pub archived_uri: Option<String>,

    #[serde(rename = "ArchivedMD5")]
    pub archived_md5: Option<String>,

    #[serde(rename = "ArchivedSize")]
    pub archived_size: Option<u64>,

    #[serde(rename = "Processing")]
    pub is_processing: bool,

    #[serde(rename = "IsVideo")]
    pub is_video: bool,

    #[serde(rename = "Hidden")]
    pub is_hidden: bool,

    #[serde(default, rename = "Watermarked")]
    pub is_watermarked: bool,

    // Album specific fields
    #[serde(rename = "DateTimeUploaded")]
    pub date_created: DateTime<Utc>,

    #[serde(rename = "LastUpdated")]
    pub last_updated: DateTime<Utc>,
}

impl Image {
    /// Returns information for the image at the provided full url
    pub async fn from_url(client: Arc<Client>, url: &str) -> Result<Self, SmugMugError> {
        let params = vec![("_verbosity", "1")];
        client
            .get::<ImageResponse>(url, Some(&params))
            .await?
            .payload
            .ok_or(SmugMugError::ResponseMissing())
            .map(|mut v| {
                v.image.client = client;
                v.image
            })
    }

    /// Returns information for the specified image id
    pub async fn from_id(client: Arc<Client>, id: &str) -> Result<Self, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?
            .join("/api/v2/image/")?
            .join(id)?;
        Self::from_url(client, req_url.as_str()).await
    }
}

// Expected response for a request to get an Image
#[derive(Deserialize, Debug)]
struct ImageResponse {
    #[serde(rename = "Image")]
    image: Image,
}
