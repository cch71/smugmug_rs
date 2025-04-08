/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::errors::SmugMugError;
use crate::v2::macros::{obj_from_url, objs_from_id_slice, stream_children_from_url};
use crate::v2::parsers::{from_privacy, is_none_or_empty_str};
use crate::v2::{Client, Image, Pages, PrivacyLevel, API_ORIGIN};
use async_stream::try_stream;
use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// Holds information returned from the Album API.
///
/// See [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/reference/album.html) for more
/// details on the individual fields.
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct Album {
    // Common to Node and Album types
    #[serde(skip)]
    pub(crate) client: Option<Client>,

    #[serde(rename = "Uri")]
    pub uri: String,

    #[serde(rename = "AlbumKey")]
    pub album_key: String,

    #[serde(rename = "AllowDownloads")]
    pub do_allow_downloads: bool,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Description", skip_serializing_if = "is_none_or_empty_str")]
    pub description: Option<String>,

    #[serde(rename = "PasswordHint", skip_serializing_if = "is_none_or_empty_str")]
    pub password_hint: Option<String>,

    #[serde(rename = "UrlName")]
    pub url_name: String,

    #[serde(rename = "WebUri")]
    pub web_uri: String,

    // #[serde(rename = "WorldSearchable", skip_serializing_if = "is_none_or_empty_str")]
    // pub is_world_searchable: Option<String>,

    // #[serde(rename = "SmugSearchable", skip_serializing_if = "is_none_or_empty_str")]
    // pub is_smug_searchable: Option<String>,

    #[serde(rename = "UploadKey", skip_serializing_if = "is_none_or_empty_str")]
    pub upload_key: Option<String>,

    #[serde(rename = "ImageCount")]
    pub image_count: u64,

    #[serde(rename = "TotalSizes")]
    pub total_sizes: Option<u64>,

    #[serde(rename = "OriginalSizes")]
    pub original_sizes: Option<u64>,

    #[serde(
        default,
        rename = "Privacy",
        deserialize_with = "from_privacy",
        skip_serializing_if = "Option::is_none"
    )]
    pub privacy: Option<PrivacyLevel>,

    // Album specific fields
    #[serde(rename = "Date", skip_serializing_if = "Option::is_none")]
    pub date_created: Option<DateTime<Utc>>,

    #[serde(rename = "ImagesLastUpdated")]
    pub images_last_updated: DateTime<Utc>,

    #[serde(rename = "LastUpdated")]
    pub last_updated: DateTime<Utc>,

    #[serde(rename = "Uris")]
    uris: AlbumUris,
}

impl Album {
    const BASE_URI: &'static str = "/api/v2/album/";

    /// Returns information for the album at the provided full url
    pub async fn from_url(client: Client, url: &str) -> Result<Self, SmugMugError> {
        obj_from_url!(client, url, AlbumResponse, album)
    }

    /// Returns information for the specified album id
    pub async fn from_id(client: Client, id: &str) -> Result<Self, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?
            .join(Self::BASE_URI)?
            .join(id)?;
        Self::from_url(client, req_url.as_str()).await
    }

    /// Returns information for the list of albums
    pub async fn from_id_slice(
        client: Client,
        id_list: &[&str],
    ) -> Result<Vec<Self>, SmugMugError> {
        objs_from_id_slice!(client, id_list, Self::BASE_URI, AlbumsResponse, albums)
    }

    /// Retrieves information about the images associated with this Album
    pub fn images(&self) -> Result<impl Stream<Item=Result<Image, SmugMugError>>, SmugMugError> {
        self.images_with_client(
            self.client.as_ref().ok_or(SmugMugError::ClientNotFound()).unwrap().clone())
    }

    /// Retrieves information about images associated with this Album using the provided client
    pub fn images_with_client(
        &self,
        client: Client,
    ) -> Result<impl Stream<Item=Result<Image, SmugMugError>>, SmugMugError> {
        // Build up the query parameters
        let params: Vec<(&str, &str)> = Vec::new();

        Ok(stream_children_from_url!(
            client,
            self.uris.album_images.as_ref(),
            &params,
            AlbumImagesResponse,
            images
        ))
    }

    async fn update_upload_key_with_client(&self, client: Client, data: Vec<u8>) -> Result<Album, SmugMugError> {
        let params = vec![("_verbosity", "1")];
        let req_url = url::Url::parse(API_ORIGIN)?.join(self.uri.as_str())?;
        client
            .patch::<AlbumResponse>(req_url.as_str(), data, Some(&params))
            .await?
            .payload
            .ok_or(SmugMugError::ResponseMissing())
            .map(|mut v| {
                v.album.client = Some(client.clone());
                v.album
            })
    }

    /// Clear the upload key on this Album with the provided client
    pub async fn clear_upload_key_with_client(&self, client: Client) -> Result<Album, SmugMugError> {
        let data = serde_json::to_vec(&json!({"UploadKey": ""}))?;
        self.update_upload_key_with_client(client, data).await
    }

    /// Clear the upload key on this Album
    pub async fn clear_upload_key(&self) -> Result<Album, SmugMugError> {
        let client = self.client.as_ref().ok_or(SmugMugError::ClientNotFound())?.clone();
        let data = serde_json::to_vec(&json!({"UploadKey": ""}))?;
        self.update_upload_key_with_client(client, data).await
    }

    /// Set the upload key for this Album
    pub async fn set_upload_key_with_client(&self, client: Client, upload_key: &str) -> Result<Album, SmugMugError> {
        let data = serde_json::to_vec(&json!({"UploadKey": upload_key}))?;
        self.update_upload_key_with_client(client, data).await
    }

    /// Set the upload key for this Album
    pub async fn set_upload_key(&self, upload_key: &str) -> Result<Album, SmugMugError> {
        let client = self.client.as_ref().ok_or(SmugMugError::ClientNotFound())?.clone();
        self.set_upload_key_with_client(client, upload_key).await
    }
}

impl PartialEq for Album {
    fn eq(&self, other: &Self) -> bool {
        self.album_key == other.album_key
    }
}
impl Eq for Album {}

impl Hash for Album {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(self.album_key.as_bytes());
        let _ = state.finish();
    }
}

impl PartialOrd for Album {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.album_key.cmp(&other.album_key))
    }
}

impl Ord for Album {
    fn cmp(&self, other: &Self) -> Ordering {
        self.album_key.cmp(&other.album_key)
    }
}

impl std::fmt::Display for Album {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "name: {}, id: {}", self.name, self.album_key)
    }
}

// Uris returned for a Node
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
struct AlbumUris {
    #[serde(rename = "AlbumImages")]
    album_images: Option<String>,
    // #[serde(rename = "User")]
    // user: String,

    // #[serde(rename = "Node")]
    // node: Option<String>,

    // #[serde(rename = "HighlightImage")]
    // highlight_image: String,
}

/// Properties that can be used in the creation of an Album
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
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

// Expected response from a User request
#[derive(Deserialize, Debug)]
struct AlbumsResponse {
    #[serde(rename = "Album")]
    albums: Vec<Album>,
}
// Expected response for a request to get an Album's images
#[derive(Deserialize, Debug)]
struct AlbumImagesResponse {
    #[serde(rename = "AlbumImage")]
    images: Vec<Image>,

    #[serde(rename = "Pages")]
    pages: Option<Pages>,
}
