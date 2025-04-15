/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::errors::SmugMugError;
use crate::v2::macros::{obj_from_url, obj_update_from_uri, obj_update_from_url, objs_from_id_slice};
use crate::v2::{Client, API_ORIGIN};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// Holds information returned from the AlbumImage/Image API.
///
/// See [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/reference/image.html) for more
/// details on the individual fields.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Image {
    #[serde(skip)]
    pub(crate) client: Option<Client>,

    #[serde(rename = "Uri")]
    pub uri: String,

    #[serde(rename = "Title")]
    pub name: String,

    #[serde(rename = "Caption", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "Altitude")]
    pub altitude: u64,

    #[serde(rename = "Latitude", skip_serializing_if = "Option::is_none")]
    pub latitude: Option<String>,

    #[serde(rename = "Longitude", skip_serializing_if = "Option::is_none")]
    pub longitude: Option<String>,

    #[serde(rename = "Format")]
    pub format: String,

    #[serde(rename = "FileName")]
    pub file_name: String,

    #[serde(rename = "ImageKey")]
    pub image_key: String,

    #[serde(rename = "KeywordArray")]
    pub keywords: Vec<String>,

    #[serde(rename = "ArchivedUri", skip_serializing_if = "Option::is_none")]
    pub archived_uri: Option<String>,

    #[serde(rename = "ArchivedMD5", skip_serializing_if = "Option::is_none")]
    pub archived_md5: Option<String>,

    #[serde(rename = "ArchivedSize", skip_serializing_if = "Option::is_none")]
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
    const BASE_URI: &'static str = "/api/v2/image/";

    /// Returns information for the image at the provided full url
    pub async fn from_url(client: Client, url: &str) -> Result<Self, SmugMugError> {
        obj_from_url!(client, url, ImageResponse, image)
    }

    /// Returns information for the specified image id
    pub async fn from_id(client: Client, id: &str) -> Result<Self, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?
            .join(Self::BASE_URI)?
            .join(id)?;
        Self::from_url(client, req_url.as_str()).await
    }

    /// Returns information for the list of image id
    pub async fn from_id_slice(
        client: Client,
        id_list: &[&str],
    ) -> Result<Vec<Self>, SmugMugError> {
        objs_from_id_slice!(client, id_list, Self::BASE_URI, ImagesResponse, images)
    }

    /// Updates this Image data fields
    pub async fn update_image_data_with_client(&self, client: Client, data: Vec<u8>) -> Result<Image, SmugMugError> {
        obj_update_from_uri!(client, self.uri.as_str(), data, ImageResponse, image)
    }

    /// Updates data for the provided image id using the given client
    pub async fn update_image_data_with_client_from_id(client: Client, data: Vec<u8>, id: &str) -> Result<Image, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?
            .join(Self::BASE_URI)?
            .join(id)?;
        obj_update_from_url!(client, req_url.as_str(), data, ImageResponse, image)
    }

    /// Retrieves the image data found at the archive uri
    pub async fn get_archive_with_client(&self, client: Client) -> Result<Bytes, SmugMugError> {
        match self.archived_uri.as_ref() {
            Some(archived_uri) => Ok(
                client
                    .get_binary_data(archived_uri, None)
                    .await?
                    .payload
                    .unwrap()),
            None => Err(SmugMugError::ImageArchiveNotFound(
                self.file_name.clone(),
                self.image_key.clone(),
            )),
        }
    }

    pub async fn get_archive(&self) -> Result<Bytes, SmugMugError> {
        self.get_archive_with_client(
            self.client.as_ref().ok_or(SmugMugError::ClientNotFound()).unwrap().clone()).await
    }
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.image_key == other.image_key
    }
}
impl Eq for Image {}

impl Hash for Image {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(self.image_key.as_bytes());
        let _ = state.finish();
    }
}

impl PartialOrd for Image {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.image_key.cmp(&other.image_key))
    }
}

impl Ord for Image {
    fn cmp(&self, other: &Self) -> Ordering {
        self.image_key
            .cmp(&other.image_key)
    }
}

impl std::fmt::Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "name: {}, filename: {} id: {}", self.name, self.file_name, self.image_key)
    }
}
// Expected response for a request to get an Image
#[derive(Deserialize, Debug)]
struct ImageResponse {
    #[serde(rename = "Image")]
    image: Image,
}
// Expected response for a request to get Images
#[derive(Deserialize, Debug)]
struct ImagesResponse {
    #[serde(rename = "Image")]
    images: Vec<Image>,
}
