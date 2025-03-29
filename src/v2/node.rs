/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::errors::SmugMugError;
use crate::v2::parsers::{from_node_type, from_privacy};
use crate::v2::{
    API_ORIGIN, Album, AlbumResponse, Client, CreateAlbumProps, NUM_TO_GET, NUM_TO_GET_STRING,
    NodeType, NodeTypeFilters, PrivacyLevel, SortDirection, SortMethod,
};
use async_stream::try_stream;
use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Holds information returned from the Node API.
///
/// See [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/reference/node.html) for more
/// details on the individual fields.
#[derive(Deserialize, Serialize, Debug)]
pub struct Node {
    // Common to Node and Album types
    #[serde(skip)]
    pub(crate) client: Arc<Client>,

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
    pub is_smug_searchable: Option<String>,

    #[serde(default, rename = "Privacy", deserialize_with = "from_privacy")]
    pub privacy: Option<PrivacyLevel>,

    // Node Specific
    // TODO: Use an ENUM
    #[serde(rename = "WorldSearchable")]
    pub is_world_searchable: Option<String>,

    #[serde(rename = "HasChildren")]
    pub has_children: bool,

    #[serde(rename = "IsRoot")]
    pub is_root: bool,

    #[serde(rename = "Type", deserialize_with = "from_node_type")]
    pub node_type: NodeType,

    #[serde(rename = "DateAdded")]
    pub date_created: DateTime<Utc>,

    #[serde(rename = "DateModified")]
    pub date_modified: DateTime<Utc>,

    #[serde(skip_serializing, rename = "Uris")]
    uris: NodeUris,
}

impl Node {
    /// Returns information for the node at the provided full url
    pub async fn from_url(client: Arc<Client>, url: &str) -> Result<Self, SmugMugError> {
        let params = vec![("_verbosity", "1")];
        client
            .get::<NodeResponse>(url, Some(&params))
            .await?
            .payload
            .ok_or(SmugMugError::ResponseMissing())
            .map(|mut v| {
                v.node.client = client;
                v.node
            })
    }

    /// Returns information for the specified node id
    pub async fn from_id(client: Arc<Client>, id: &str) -> Result<Self, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?
            .join("/api/v2/node/")?
            .join(id)?;
        Self::from_url(client, req_url.as_str()).await
    }

    /// Retrieves the Album specific information about this Node
    pub async fn album(&self) -> Result<Album, SmugMugError> {
        let album_uri = self.uris.album.as_ref().ok_or(SmugMugError::NotAnAlbum())?;
        let req_url = url::Url::parse(API_ORIGIN)?.join(album_uri)?;

        Album::from_url(self.client.clone(), req_url.as_str()).await
    }

    /// Creates album off this node
    pub async fn create_album(&self, album_props: CreateAlbumProps) -> Result<Album, SmugMugError> {
        let children_uri = self.uris.child_nodes.as_ref().unwrap(); //Should always be true right?
        let req_url = url::Url::parse(API_ORIGIN)?.join(children_uri)?;
        let params = vec![("_verbosity", "1")];

        let data = serde_json::to_vec(&album_props)?;

        self.client
            .post::<AlbumResponse>(req_url.as_str(), data, Some(&params))
            .await?
            .payload
            .ok_or(SmugMugError::ResponseMissing())
            .map(|mut v| {
                v.album.client = self.client.clone();
                v.album
            })
    }

    /// Retrieves the Child Nodes information for this Node
    pub fn children(
        &self,
        filter_by_type: NodeTypeFilters,
        sort_direction: SortDirection,
        sort_method: SortMethod,
    ) -> impl Stream<Item = Result<Node, SmugMugError>> {
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

                let nodes = self.client.get::<NodesResponse>(
                    req_url.as_str(), Some(&params)
                ).await?
                .payload
                .ok_or(SmugMugError::ResponseMissing())?
                .nodes;

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
