/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::errors::SmugMugError;
use crate::v2::parsers::{from_node_type, from_privacy};
use crate::v2::{Album, AlbumResponse, ApiClient, CreateAlbumProps, NodeType, NodeTypeFilters, PrivacyLevel, SortDirection, SortMethod, API_ORIGIN, NUM_TO_GET, NUM_TO_GET_STRING};
use async_stream::try_stream;
use chrono::Utc;
use futures::Stream;
use serde::Deserialize;
use std::sync::Arc;

/// Holds information returned from the Node API
#[derive(Deserialize, Debug)]
pub struct Node {
    // Common to Node and Album types
    #[serde(skip)]
    pub(crate) api_client: Arc<ApiClient>,

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

    /// Creates album off this node
    pub async fn create_album(&self, album_props: CreateAlbumProps) -> Result<Album, SmugMugError> {
        let children_uri = self.uris.child_nodes.as_ref().unwrap(); //Should always be true right?
        let req_url = url::Url::parse(API_ORIGIN)?.join(children_uri)?;
        let params = vec![("_verbosity", "1")];

        let data = serde_json::to_vec(&album_props)?;

        self
            .api_client
            .post::<AlbumResponse>(req_url.as_str(), data, Some(&params))
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

// Expected response from a Node request
#[derive(Deserialize, Debug)]
pub(crate) struct NodeResponse {
    #[serde(rename = "Node")]
    pub(crate) node: Node,
}

// Expected response from a Node Children request
#[derive(Deserialize, Debug)]
pub(crate) struct NodesResponse {
    #[serde(rename = "Node")]
    pub(crate) nodes: Vec<Node>,
}
