#![feature(async_stream)]

use std::str::FromStr;
use crate::errors::SmugMugError;
use crate::v2::{API_ORIGIN, ApiClient, ApiParams};
use async_stream::try_stream;
use chrono::Utc;
use futures::{
    Stream,
    stream::{StreamExt},
};
use serde::Deserialize;
use std::sync::Arc;
use const_format::formatcp;
use strum_macros::{EnumString, IntoStaticStr};

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
#[derive(Deserialize, Debug)]
pub struct ResponseBody<ResponseType> {
    // #[serde(rename = "Code")]
    // code: u32,

    // #[serde(rename = "Message")]
    // message: String,
    #[serde(rename = "Response")]
    pub response: ResponseType,
}

#[derive(Deserialize, Debug)]
pub struct UserResponse {
    #[serde(rename = "User")]
    pub user: User,
}

#[derive(Deserialize, Debug)]
pub struct NodeResponse {
    #[serde(rename = "Node")]
    pub node: Node,
}

#[derive(Deserialize, Debug)]
pub struct NodesResponse {
    #[serde(rename = "Node")]
    pub nodes: Vec<Node>,
}

#[derive(Deserialize, Debug)]
pub struct User {
    #[serde(skip)]
    api_client: Arc<ApiClient>,

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
        let params = vec![("_verbosity", "1")];
        let req_url = url::Url::parse(API_ORIGIN)?.join(self.uris.node.as_str())?;
        let mut node_info = self
            .api_client
            .node_info::<ResponseBody<NodeResponse>>(req_url.as_str(), Some(&params))
            .await?
            .response
            .node;
        node_info.api_client = self.api_client.clone();
        Ok(node_info)
    }
}

#[derive(Deserialize, Debug)]
struct UserUris {
    #[serde(rename = "Node")]
    node: String,

    #[serde(rename = "Features")]
    features: String,

    #[serde(rename = "UserProfile")]
    user_profile: String,

    #[serde(rename = "UserAlbums")]
    user_albums: String,

    #[serde(rename = "SiteSettings")]
    site_settings: String,
}

#[derive(Deserialize, Debug)]
pub struct Node {
    #[serde(skip)]
    api_client: Arc<ApiClient>,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Description")]
    pub description: String,

    #[serde(rename = "HasChildren")]
    pub has_children: bool,

    #[serde(rename = "IsRoot")]
    pub is_root: bool,

    // TODO: Use an ENUM
    #[serde(rename = "WorldSearchable")]
    pub is_world_searchable: String,

    // TODO: Use an ENUM
    #[serde(rename = "SmugSearchable")]
    pub is_smug_searchable: String,

    #[serde(rename = "Privacy", deserialize_with = "from_privacy")]
    pub privacy: PrivacyLevel,

    #[serde(rename = "UrlName")]
    pub url_name: String,

    #[serde(rename = "PasswordHint")]
    pub password_hint: String,

    #[serde(rename = "WebUri")]
    pub web_uri: String,

    #[serde(rename = "Type", deserialize_with = "from_node_type")]
    pub node_type: NodeType,

    #[serde(rename = "DateAdded")]
    pub date_added: chrono::DateTime<Utc>,

    #[serde(rename = "DateModified")]
    pub date_modified: chrono::DateTime<Utc>,

    #[serde(rename = "Uris")]
    uris: NodeUris,
}

fn from_privacy<'de, D>(deserializer: D) -> Result<PrivacyLevel, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    PrivacyLevel::from_str(&s).or_else(PrivacyLevel::Unknown)
}
fn from_node_type<'de, D>(deserializer: D) -> Result<NodeType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    NodeType::from_str(&s).or(Ok(NodeType::Unknown))
}

#[derive(Deserialize, Debug)]
struct NodeUris {
    #[serde(rename = "ChildNodes")]
    child_nodes: Option<String>,

    #[serde(rename = "ParentNode")]
    parent_node: Option<String>,

    #[serde(rename = "ParentNodes")]
    parent_nodes: String,

    #[serde(rename = "User")]
    user: String,

    // Only present if node is an album type
    #[serde(rename = "Album")]
    albums: Option<String>,

    #[serde(rename = "HighlightImage")]
    highlight_image: String,
}

impl Node {
    pub fn children(
        &self,
        filter_by_type: NodeTypeFilters,
        sort_direction: SortDirection,
        sort_method: SortMethod,
    ) -> impl Stream<Item = Result<Node, SmugMugError>> {
        const NUM_TO_GET: usize = 3;
        const NUM_TO_GET_STRING: &str = formatcp!("{}",NUM_TO_GET);
        let mut params = vec![
            ("_verbosity", "1"),
            ("count", NUM_TO_GET_STRING),
            ("SortDirection", sort_direction.into())
        ];
        match filter_by_type {
            NodeTypeFilters::Any => (),
            _ => params.push(("Type", filter_by_type.into())),
        };

        match sort_method {
            SortMethod::SortIndex => (),
            _ => params.push(("SortMethod", sort_method.into())),
        }

        let mut start_idx = 0;
        try_stream! {
            let mut params = params.clone();
            let start_idx_str = start_idx.to_string();
            params.push(("start", start_idx_str.as_str()));

            let req_url = match self.uris.child_nodes.as_ref() {
                Some(child_nodes) => url::Url::parse(API_ORIGIN)?.join(child_nodes.as_str())?,
                None => return,
            };

            loop {
                let nodes = self.api_client.node_children::<ResponseBody<NodesResponse>>(
                    req_url.as_str(), Some(&params)
                ).await?.response.nodes;

                let is_done = nodes.len() != NUM_TO_GET;

                for node in nodes {
                    yield node
                }

                if is_done {
                    break;
                }
                start_idx += NUM_TO_GET;
                println!("!!! Would get more");
                break;
            }
        }
    }
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

#[derive(Debug, EnumString)]
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

    /// Returns information for the authenticated user
    pub async fn authenticated_user_info(&self) -> Result<User, SmugMugError> {
        let params = vec![("_verbosity", "1")];
        let mut user_info = self
            .api_client
            .authenticated_user_info::<ResponseBody<UserResponse>>(Some(&params))
            .await?
            .response
            .user;
        user_info.api_client = self.api_client.clone();
        Ok(user_info)
    }
}
