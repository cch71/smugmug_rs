/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::errors::SmugMugError;
use crate::v2::macros::{
    obj_from_url, obj_update_from_uri, obj_update_from_url, objs_from_id_slice,
    stream_children_from_url,
};
use crate::v2::parsers::{from_node_type, from_privacy, is_none_or_empty_str};
use crate::v2::{
    Album, Client, CreateAlbumProps, NodeType, NodeTypeFilters, Pages, PrivacyLevel, SortDirection,
    SortMethod, API_ORIGIN,
};
use async_stream::try_stream;
use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// Holds information returned from the Node API.
///
/// See [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/reference/node.html) for more
/// details on the individual fields.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Node {
    // Common to Node and Album types
    #[serde(skip)]
    pub(crate) client: Option<Client>,

    #[serde(rename = "Uri")]
    pub uri: String,

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

    // #[serde(rename = "SmugSearchable", skip_serializing_if = "is_none_or_empty_str")]
    // pub is_smug_searchable: Option<String>,

    // #[serde(rename = "WorldSearchable", skip_serializing_if = "is_none_or_empty_str")]
    // pub is_world_searchable: Option<String>,
    #[serde(
        default,
        rename = "Privacy",
        deserialize_with = "from_privacy",
        skip_serializing_if = "Option::is_none"
    )]
    pub privacy: Option<PrivacyLevel>,

    #[serde(rename = "HasChildren")]
    pub has_children: bool,

    #[serde(rename = "IsRoot")]
    pub is_root: bool,

    #[serde(rename = "NodeID")]
    pub node_id: String,

    #[serde(rename = "Type", deserialize_with = "from_node_type")]
    pub node_type: NodeType,

    #[serde(rename = "DateAdded")]
    pub date_created: DateTime<Utc>,

    #[serde(rename = "DateModified")]
    pub date_modified: DateTime<Utc>,

    #[serde(rename = "Uris")]
    uris: NodeUris,
}

impl Node {
    const BASE_URI: &'static str = "/api/v2/node/";

    /// Returns information for the node at the provided full url
    pub async fn from_url(client: Client, url: &str) -> Result<Self, SmugMugError> {
        obj_from_url!(client, url, NodeResponse, node)
    }

    /// Returns information for the specified node id using the provided client
    pub async fn from_id(client: Client, id: &str) -> Result<Self, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?
            .join(Self::BASE_URI)?
            .join(id)?;
        Self::from_url(client, req_url.as_str()).await
    }

    /// Returns information for the list of node id
    pub async fn from_id_slice(
        client: Client,
        id_list: &[&str],
    ) -> Result<Vec<Self>, SmugMugError> {
        objs_from_id_slice!(client, id_list, Self::BASE_URI, NodesResponse, nodes)
    }

    /// Updates this nodes data fields
    pub async fn update_node_data_with_client(
        &self,
        client: Client,
        data: Vec<u8>,
    ) -> Result<Node, SmugMugError> {
        obj_update_from_uri!(client, self.uri.as_str(), data, NodeResponse, node)
    }

    /// Updates data for the provided Node id using the given client
    pub async fn update_node_data_with_client_from_id(
        client: Client,
        data: Vec<u8>,
        id: &str,
    ) -> Result<Node, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?
            .join(Self::BASE_URI)?
            .join(id)?;
        obj_update_from_url!(client, req_url.as_str(), data, NodeResponse, node)
    }

    /// Retrieves the Album specific information about this Node
    pub async fn album(&self) -> Result<Album, SmugMugError> {
        let album_uri = self.uris.album.as_ref().ok_or(SmugMugError::NotAnAlbum())?;
        let req_url = url::Url::parse(API_ORIGIN)?.join(album_uri)?;

        Album::from_url(
            self.client
                .as_ref()
                .ok_or(SmugMugError::ClientNotFound())?
                .clone(),
            req_url.as_str(),
        )
            .await
    }

    /// Retrieves the album id if this node is an [`Album`] type
    pub fn album_id(&self) -> Result<String, SmugMugError> {
        let album_uri = self.uris.album.as_ref().ok_or(SmugMugError::NotAnAlbum())?;
        let req_url = url::Url::parse(API_ORIGIN)?.join(album_uri)?;
        let album_id_segment = req_url
            .path_segments()
            .ok_or(SmugMugError::NotAnAlbum())?
            .next_back()
            .ok_or(SmugMugError::NotAnAlbum())?;
        Ok(album_id_segment.to_string())
    }

    /// Creates album off this node using the given client
    pub async fn create_album_with_client(
        &self,
        client: Client,
        album_props: CreateAlbumProps,
    ) -> Result<Album, SmugMugError> {
        let children_uri = self.uris.child_nodes.as_ref().unwrap(); //Should always be true right?
        let req_url = url::Url::parse(API_ORIGIN)?.join(children_uri)?;
        let params = vec![("_verbosity", "1")];

        let mut album_props: serde_json::Value = serde_json::to_value(&album_props)?;
        album_props
            .as_object_mut()
            .ok_or(SmugMugError::JsonSerialization(
                "Album Props is not a JSON object".to_string(),
            ))?
            .insert("Type".to_string(), json!("Album"));
        let data = serde_json::to_vec(&album_props)?;

        let node = client
            .post::<NodeResponse>(req_url.as_str(), data, Some(&params))
            .await?
            .payload
            .ok_or(SmugMugError::ResponseMissing())
            .map(|mut v| {
                v.node.client = Some(client.clone());
                v.node
            })?;
        node.album().await
    }

    /// Creates an album off this node
    pub async fn create_album(&self, album_props: CreateAlbumProps) -> Result<Album, SmugMugError> {
        let client = self
            .client
            .as_ref()
            .ok_or(SmugMugError::ClientNotFound())?
            .clone();
        self.create_album_with_client(client, album_props).await
    }

    /// Retrieves the Child Nodes information for this Node
    pub fn children(
        &self,
        filter_by_type: NodeTypeFilters,
        sort_direction: SortDirection,
        sort_method: SortMethod,
    ) -> Result<impl Stream<Item=Result<Node, SmugMugError>>, SmugMugError> {
        self.children_with_client(
            self.client
                .as_ref()
                .ok_or(SmugMugError::ClientNotFound())?
                .clone(),
            filter_by_type,
            sort_direction,
            sort_method,
        )
    }

    /// Retrieves the child nodes information of this node using the provided client
    pub fn children_with_client(
        &self,
        client: Client,
        filter_by_type: NodeTypeFilters,
        sort_direction: SortDirection,
        sort_method: SortMethod,
    ) -> Result<impl Stream<Item=Result<Node, SmugMugError>>, SmugMugError> {
        // Build up the query parameters
        let mut params: Vec<(&str, &str)> = vec![("SortDirection", sort_direction.into())];
        match filter_by_type {
            NodeTypeFilters::Any => (),
            _ => params.push(("Type", filter_by_type.into())),
        };

        match sort_method {
            SortMethod::SortIndex => (),
            _ => params.push(("SortMethod", sort_method.into())),
        }

        Ok(stream_children_from_url!(
            client,
            self.uris.child_nodes.as_ref(),
            &params,
            NodesResponse,
            nodes
        ))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
    }
}
impl Eq for Node {}

impl Hash for Node {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(self.node_id.as_bytes());
        let _ = state.finish();
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.node_id.cmp(&other.node_id)
    }
}
impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "name: {}, id: {}", self.name, self.node_id)
    }
}

// Uris returned for a Node
#[derive(Serialize, Deserialize, Debug, Clone)]
struct NodeUris {
    #[serde(rename = "ChildNodes", skip_serializing_if = "Option::is_none")]
    child_nodes: Option<String>,

    // #[serde(rename = "ParentNode")]
    // parent_node: Option<String>,

    // #[serde(rename = "ParentNodes")]
    // parent_nodes: String,

    // #[serde(rename = "User")]
    // user: String,

    // Only present if node is an album type
    #[serde(rename = "Album", skip_serializing_if = "Option::is_none")]
    album: Option<String>,
    // #[serde(rename = "HighlightImage")]
    // highlight_image: String,
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

    #[serde(rename = "Pages")]
    pages: Option<Pages>,
}
