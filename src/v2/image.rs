/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::Client;
use chrono::Utc;
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

    #[serde(rename = "IsVideo")]
    pub is_video: bool,

    #[serde(rename = "Hidden")]
    pub is_hidden: bool,

    #[serde(rename = "Watermarked")]
    pub is_watermarked: bool,

    // Album specific fields
    #[serde(rename = "DateTimeUploaded")]
    pub date_created: chrono::DateTime<Utc>,

    #[serde(rename = "LastUpdated")]
    pub last_updated: chrono::DateTime<Utc>,

    // #[serde(rename = "Uris")]
    // uris: ImageUris,
}

// Uris returned for an Image/AlbumImage
// #[derive(Deserialize, Debug)]
// struct ImageUris {
//     #[serde(rename = "ImageSizeDetails")]
//     image_size_details: String,
// }

// Expected response for a request to get an Album's images
#[derive(Deserialize, Debug)]
pub(crate) struct AlbumImagesResponse {
    #[serde(rename = "AlbumImage")]
    pub(crate) images: Vec<Image>,
}
