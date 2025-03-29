/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::errors::SmugMugError;
use crate::v2::parsers::{from_empty_str_to_none, from_privacy};
use crate::v2::{API_ORIGIN, Client, Image, NUM_TO_GET, NUM_TO_GET_STRING, PrivacyLevel};
use async_stream::try_stream;
use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

/// Holds information returned from the Album API.
///
/// See [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/reference/album.html) for more
/// details on the individual fields.
#[derive(Serialize, Deserialize, Debug)]
pub struct Album {
    // Common to Node and Album types
    #[serde(skip)]
    pub(crate) client: Arc<Client>,

    #[serde(rename = "Uri")]
    pub uri: String,

    #[serde(rename = "AlbumKey")]
    pub album_key: String,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Description")]
    pub description: String,

    #[serde(rename = "PasswordHint")]
    pub password_hint: String,

    #[serde(rename = "UrlName")]
    pub url_name: String,

    #[serde(rename = "WebUri")]
    pub web_uri: String,

    #[serde(rename = "WorldSearchable")]
    pub is_world_searchable: Option<bool>,

    // TODO: Use an ENUM
    #[serde(rename = "SmugSearchable")]
    pub is_smug_searchable: Option<String>,

    #[serde(
        default,
        rename = "UploadKey",
        deserialize_with = "from_empty_str_to_none"
    )]
    pub upload_key: Option<String>,

    #[serde(rename = "ImageCount")]
    pub image_count: u64,

    #[serde(default, rename = "Privacy", deserialize_with = "from_privacy")]
    pub privacy: Option<PrivacyLevel>,

    // Album specific fields
    #[serde(rename = "Date")]
    pub date_created: Option<DateTime<Utc>>,

    #[serde(rename = "ImagesLastUpdated")]
    pub images_last_updated: DateTime<Utc>,

    #[serde(rename = "LastUpdated")]
    pub last_updated: DateTime<Utc>,

    #[serde(skip_serializing, rename = "Uris")]
    uris: AlbumUris,
}

impl Album {
    /// Returns information for the album at the provided full url
    pub async fn from_url(client: Arc<Client>, url: &str) -> Result<Self, SmugMugError> {
        let params = vec![("_verbosity", "1")];
        client
            .get::<AlbumResponse>(url, Some(&params))
            .await?
            .payload
            .ok_or(SmugMugError::ResponseMissing())
            .map(|mut v| {
                v.album.client = client;
                v.album
            })
    }

    /// Returns information for the specified album id
    pub async fn from_id(client: Arc<Client>, id: &str) -> Result<Self, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?
            .join("/api/v2/album/")?
            .join(id)?;
        Self::from_url(client, req_url.as_str()).await
    }

    /// Retrieves information about the images associated with this Album
    pub fn images(&self) -> impl Stream<Item = Result<Image, SmugMugError>> {
        // Build up the query parameters
        let params = vec![("_verbosity", "1"), ("count", NUM_TO_GET_STRING)];

        // Page through and retrieve the nodes and return them as a stream.
        try_stream! {
            let mut start_idx = 0;

            let req_url = url::Url::parse(API_ORIGIN)?.join(self.uris.album_images.as_str())?;

            loop {
                let mut params = params.clone();
                let start_idx_str = start_idx.to_string();
                params.push(("start", start_idx_str.as_str()));

                let nodes = self.client.get::<AlbumImagesResponse>(
                    req_url.as_str(), Some(&params)
                ).await?
                .payload
                .ok_or(SmugMugError::ResponseMissing())?
                .images;

                let is_done = nodes.len() != NUM_TO_GET;

                for mut node in nodes {
                    node.client = self.client.clone();
                    yield node
                }

                if is_done {
                    break;
                }
                start_idx += NUM_TO_GET;
            }
        }
    }

    async fn update_upload_key(&self, data: Vec<u8>) -> Result<Album, SmugMugError> {
        let params = vec![("_verbosity", "1")];
        let req_url = url::Url::parse(API_ORIGIN)?.join(self.uri.as_str())?;
        self.client
            .patch::<AlbumResponse>(req_url.as_str(), data, Some(&params))
            .await?
            .payload
            .ok_or(SmugMugError::ResponseMissing())
            .map(|mut v| {
                v.album.client = self.client.clone();
                v.album
            })
    }

    /// Clear the upload key on this Album
    pub async fn clear_upload_key(&self) -> Result<Album, SmugMugError> {
        let data = serde_json::to_vec(&json!({"UploadKey": ""}))?;
        self.update_upload_key(data).await
    }

    /// Set the upload key for this Album
    pub async fn set_upload_key(&self, upload_key: &str) -> Result<Album, SmugMugError> {
        let data = serde_json::to_vec(&json!({"UploadKey": upload_key}))?;
        self.update_upload_key(data).await
    }
}

// Uris returned for a Node
#[derive(Deserialize, Debug)]
struct AlbumUris {
    #[serde(rename = "AlbumImages")]
    album_images: String,
    // #[serde(rename = "User")]
    // user: String,

    // #[serde(rename = "Node")]
    // node: Option<String>,

    // #[serde(rename = "HighlightImage")]
    // highlight_image: String,
}

/// Properties that can be used in the creation of an Album
#[derive(Serialize, Default, Debug)]
pub struct CreateAlbumProps {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Description")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "PasswordHint")]
    pub password_hint: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "UrlName")]
    pub url_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "WebUri")]
    pub web_uri: Option<String>,

    // #[serde(rename = "WorldSearchable")]
    // pub is_world_searchable: String,

    // #[serde(rename = "SmugSearchable")]
    // pub is_smug_searchable: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, rename = "UploadKey")]
    pub upload_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Privacy")]
    pub privacy: Option<PrivacyLevel>,
}

// Expected response for an Album request
#[derive(Deserialize, Debug)]
pub(crate) struct AlbumResponse {
    #[serde(rename = "Album")]
    pub(crate) album: Album,
}

// Expected response for a request to get an Album's images
#[derive(Deserialize, Debug)]
struct AlbumImagesResponse {
    #[serde(rename = "AlbumImage")]
    images: Vec<Image>,
}
