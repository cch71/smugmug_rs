/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */


use crate::errors::SmugMugError;
use crate::v2::{ApiClient, API_ORIGIN};
use async_stream::try_stream;
use chrono::Utc;
use const_format::formatcp;
use futures::Stream;
use serde::Deserialize;
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use strum_macros::{EnumString, IntoStaticStr};

// When retrieving pages this is the default records to retrieve
const NUM_TO_GET: usize = 25;
// String representation of default number of records to retrieve
const NUM_TO_GET_STRING: &str = formatcp!("{}", NUM_TO_GET);

/// Example
// /// ```rust
// ///     use smugmug::SmugMug;
// ///     let api_key = "".to_string();
// ///     let api_secret = "".to_string();
// ///     let token = "".to_string();
// ///     let token_secret = "".to_string();
// ///     let smugmug_client = SmugMug::new(&api_key, &api_secret, &token, &token_secret);
// ///     smugmug_client.authenticated_user_info().await.unwrap();
// /// ```
#[derive(Debug, Clone)]
pub struct Client {
    api_client: Arc<ApiClient>,
}

impl Client {
    pub fn new(
        consumer_key: &str,
        consumer_secret: &str,
        access_token: &str,
        token_secret: &str,
    ) -> Self {
        Self {
            api_client: Arc::new(ApiClient::new(
                consumer_key,
                consumer_secret,
                access_token,
                token_secret,
            )),
        }
    }

    /// Returns information for the user at the provided full url
    pub async fn user_from_url(&self, url: &str) -> Result<User, SmugMugError> {
        let params = vec![("_verbosity", "1")];
        self
            .api_client
            .get::<UserResponse>(url, Some(&params))
            .await?
            .ok_or(SmugMugError::ResponseMissing())
            .map(|mut v| {
                v.user.api_client = self.api_client.clone();
                v.user
            })
    }

    /// Returns information for the specified user id
    pub async fn user_from_id(&self, user_id: &str) -> Result<User, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?
            .join("/api/v2/user")?
            .join(user_id)?;
        self.user_from_url(req_url.as_str()).await
    }

    /// Returns information for the authenticated user
    pub async fn authenticated_user_info(&self) -> Result<User, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?.join("/api/v2!authuser")?;
        self.user_from_url(req_url.as_str()).await
    }
}

/// Holds information returned from the User API
#[derive(Deserialize, Debug)]
pub struct User {
    #[serde(skip)]
    api_client: Arc<ApiClient>,

    #[serde(rename = "Uri")]
    pub uri: String,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "FirstName")]
    pub first_name: String,

    #[serde(rename = "LastName")]
    pub last_name: String,

    #[serde(rename = "NickName")]
    pub nick_name: String,

    #[serde(rename = "Plan")]
    pub plan: String,

    #[serde(rename = "TimeZone")]
    pub time_zone: String,

    #[serde(rename = "Uris")]
    uris: UserUris,
}

impl User {
    pub async fn node(self) -> Result<Node, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?.join(self.uris.node.as_str())?;
        let params = vec![("_verbosity", "1")];
        self
            .api_client
            .get::<NodeResponse>(req_url.as_str(), Some(&params))
            .await?
            .ok_or(SmugMugError::ResponseMissing())
            .map(|mut v| {
                v.node.api_client = self.api_client.clone();
                v.node
            })
    }
}

#[derive(Deserialize, Debug)]
struct UserUris {
    #[serde(rename = "Node")]
    node: String,

    // #[serde(rename = "Features")]
    // features: String,

    // #[serde(rename = "UserProfile")]
    // user_profile: String,

    // #[serde(rename = "UserAlbums")]
    // user_albums: String,

    // #[serde(rename = "SiteSettings")]
    // site_settings: String,
}

/// Holds information returned from the Node API
#[derive(Deserialize, Debug)]
pub struct Node {
    // Common to Node and Album types
    #[serde(skip)]
    api_client: Arc<ApiClient>,

    #[serde(rename = "Uri")]
    pub uri: String,

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

    // TODO: Use an ENUM
    #[serde(rename = "SmugSearchable")]
    pub is_smug_searchable: String,

    #[serde(rename = "Privacy", deserialize_with = "from_privacy")]
    pub privacy: PrivacyLevel,

    // Node Specific
    // TODO: Use an ENUM
    #[serde(rename = "WorldSearchable")]
    pub is_world_searchable: String,

    #[serde(rename = "HasChildren")]
    pub has_children: bool,

    #[serde(rename = "IsRoot")]
    pub is_root: bool,

    #[serde(rename = "Type", deserialize_with = "from_node_type")]
    pub node_type: NodeType,

    #[serde(rename = "DateAdded")]
    pub date_created: chrono::DateTime<Utc>,

    #[serde(rename = "DateModified")]
    pub date_modified: chrono::DateTime<Utc>,

    #[serde(rename = "Uris")]
    uris: NodeUris,
}

impl Node {
    /// Retrieves the Album specific information about this Node
    pub async fn album(&self) -> Result<Album, SmugMugError> {
        let album_uri = self.uris.album.as_ref().ok_or(SmugMugError::NotAnAlbum())?;
        let req_url = url::Url::parse(API_ORIGIN)?.join(album_uri)?;
        let params = vec![("_verbosity", "1")];

        self
            .api_client
            .get::<AlbumResponse>(req_url.as_str(), Some(&params))
            .await?
            .ok_or(SmugMugError::ResponseMissing())
            .map(|mut v| {
                v.album.api_client = self.api_client.clone();
                v.album
            })
    }

    /// Retrieves the Child Nodes information for this Node
    pub fn children(
        &self,
        filter_by_type: NodeTypeFilters,
        sort_direction: SortDirection,
        sort_method: SortMethod,
    ) -> impl Stream<Item=Result<Node, SmugMugError>> {
        // Build up the query parameters
        let mut params = vec![
            ("_verbosity", "1"),
            ("count", NUM_TO_GET_STRING),
            ("SortDirection", sort_direction.into()),
        ];
        match filter_by_type {
            NodeTypeFilters::Any => (),
            _ => params.push(("Type", filter_by_type.into())),
        };

        match sort_method {
            SortMethod::SortIndex => (),
            _ => params.push(("SortMethod", sort_method.into())),
        }

        // Page through and retrieve the nodes and return them as a stream.
        try_stream! {
            let mut start_idx = 0;

            let req_url = match self.uris.child_nodes.as_ref() {
                Some(child_nodes) => url::Url::parse(API_ORIGIN)?.join(child_nodes.as_str())?,
                None => return,
            };

            loop {
                let mut params = params.clone();
                let start_idx_str = start_idx.to_string();
                params.push(("start", start_idx_str.as_str()));

                let nodes = self.api_client.get::<NodesResponse>(
                    req_url.as_str(), Some(&params)
                ).await?
                .ok_or(SmugMugError::ResponseMissing())?
                .nodes;

                let is_done = nodes.len() != NUM_TO_GET;

                for mut node in nodes {
                    node.api_client = self.api_client.clone();
                    yield node
                }

                if is_done {
                    break;
                }
                start_idx += NUM_TO_GET;
            }
        }
    }
}

// Uris returned for a Node
#[derive(Deserialize, Debug)]
struct NodeUris {
    #[serde(rename = "ChildNodes")]
    child_nodes: Option<String>,

    // #[serde(rename = "ParentNode")]
    // parent_node: Option<String>,

    // #[serde(rename = "ParentNodes")]
    // parent_nodes: String,

    // #[serde(rename = "User")]
    // user: String,

    // Only present if node is an album type
    #[serde(rename = "Album")]
    album: Option<String>,

    // #[serde(rename = "HighlightImage")]
    // highlight_image: String,
}

/// Holds information returned from the Album API
#[derive(Deserialize, Debug)]
pub struct Album {
    // Common to Node and Album types
    #[serde(skip)]
    api_client: Arc<ApiClient>,

    #[serde(rename = "Uri")]
    pub uri: String,

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
    pub is_world_searchable: bool,

    // TODO: Use an ENUM
    #[serde(rename = "SmugSearchable")]
    pub is_smug_searchable: String,

    #[serde(default, rename = "UploadKey", deserialize_with = "from_empty_str_to_none")]
    pub upload_key: Option<String>,

    #[serde(rename = "ImageCount")]
    pub image_count: u64,

    #[serde(rename = "Privacy", deserialize_with = "from_privacy")]
    pub privacy: PrivacyLevel,

    // Album specific fields
    #[serde(rename = "Date")]
    pub date_created: chrono::DateTime<Utc>,

    #[serde(rename = "ImagesLastUpdated")]
    pub images_last_updated: chrono::DateTime<Utc>,

    #[serde(rename = "LastUpdated")]
    pub last_updated: chrono::DateTime<Utc>,

    #[serde(rename = "Uris")]
    uris: AlbumUris,
}

impl Album {
    /// Retrieves information about the images for this Album
    pub fn images(&self) -> impl Stream<Item=Result<Image, SmugMugError>> {
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

                let nodes = self.api_client.get::<AlbumImagesResponse>(
                    req_url.as_str(), Some(&params)
                ).await?
                .ok_or(SmugMugError::ResponseMissing())?
                .images;

                let is_done = nodes.len() != NUM_TO_GET;

                for mut node in nodes {
                    node.api_client = self.api_client.clone();
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
        self.api_client.patch::<AlbumResponse>(req_url.as_str(), data, Some(&params))
            .await?
            .ok_or(SmugMugError::ResponseMissing())
            .map(|mut v| {
                v.album.api_client = self.api_client.clone();
                v.album
            })
    }

    /// Clear the upload key
    pub async fn clear_upload_key(&self) -> Result<Album, SmugMugError> {
        let data = serde_json::to_vec(&json!({"UploadKey": ""}))?;
        self.update_upload_key(data).await
    }

    /// Set the upload key
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

/// Holds information returned from the AlbumImage/Image API
#[derive(Deserialize, Debug)]
pub struct Image {
    #[serde(skip)]
    api_client: Arc<ApiClient>,

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

    #[serde(rename = "Uris")]
    uris: ImageUris,
}

// Uris returned for an Image/AlbumImage
#[derive(Deserialize, Debug)]
struct ImageUris {
    #[serde(rename = "ImageSizeDetails")]
    image_size_details: String,
}

#[derive(Debug, EnumString, IntoStaticStr)]
pub enum SortMethod {
    Organizer,
    SortIndex,
    Name,
    DateAdded,
    DateModified,
}

#[derive(Debug, EnumString, IntoStaticStr)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, EnumString, IntoStaticStr)]
pub enum PrivacyLevel {
    Unknown,
    Public,
    Unlisted,
    Private,
}

#[derive(Debug, EnumString, IntoStaticStr)]
pub enum NodeTypeFilters {
    Any,
    Album,
    Folder,
    Page,
    #[strum(to_string = "System Album")]
    SystemAlbum,
    #[strum(to_string = "Folder Album Page")]
    FolderAlbumPage,
}

#[derive(Debug, EnumString, IntoStaticStr)]
pub enum NodeType {
    Unknown,
    Album,
    Folder,
    Page,
    #[strum(to_string = "System Folder")]
    SystemFolder,
    #[strum(to_string = "System Page")]
    SystemPage,
}

// Expected response from a User request
#[derive(Deserialize, Debug)]
struct UserResponse {
    #[serde(rename = "User")]
    user: User,
}

// Expected response from a Node request
#[derive(Deserialize, Debug)]
struct NodeResponse {
    #[serde(rename = "Node")]
    node: Node,
}

// Expected response from a Node Children request
#[derive(Deserialize, Debug)]
struct NodesResponse {
    #[serde(rename = "Node")]
    nodes: Vec<Node>,
}

// Expected response for an Album request
#[derive(Deserialize, Debug)]
struct AlbumResponse {
    #[serde(rename = "Album")]
    album: Album,
}

// Expected response for a request to get an Album's images
#[derive(Deserialize, Debug)]
struct AlbumImagesResponse {
    #[serde(rename = "AlbumImage")]
    images: Vec<Image>,
}

// Parses node type
fn from_node_type<'de, D>(deserializer: D) -> Result<NodeType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    NodeType::from_str(&s).or(Ok(NodeType::Unknown))
}

// Parses privacy type
fn from_privacy<'de, D>(deserializer: D) -> Result<PrivacyLevel, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    PrivacyLevel::from_str(&s).or(Ok(PrivacyLevel::Unknown))
}

// Parses strings that may be "" and sets to None
fn from_empty_str_to_none
<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}