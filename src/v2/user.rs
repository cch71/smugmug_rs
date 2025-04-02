/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::errors::SmugMugError;
use crate::v2::macros::obj_from_url;
use crate::v2::parsers::is_none_or_empty_str;
use crate::v2::{Client, Node, API_ORIGIN};
use serde::{Deserialize, Serialize};

/// Holds information returned from the User API.
///
/// See [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/reference/user.html) for more
/// details on the individual fields.
#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    #[serde(skip)]
    pub(crate) client: Option<Client>,

    #[serde(rename = "Uri")]
    pub uri: String,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "FirstName", skip_serializing_if = "is_none_or_empty_str")]
    pub first_name: Option<String>,

    #[serde(rename = "LastName", skip_serializing_if = "is_none_or_empty_str")]
    pub last_name: Option<String>,

    #[serde(rename = "NickName", skip_serializing_if = "is_none_or_empty_str")]
    pub nick_name: Option<String>,

    #[serde(rename = "Plan", skip_serializing_if = "is_none_or_empty_str")]
    pub plan: Option<String>,

    #[serde(rename = "TimeZone", skip_serializing_if = "is_none_or_empty_str")]
    pub time_zone: Option<String>,

    #[serde(rename = "WebUri")]
    pub web_uri: String,

    #[serde(skip_serializing, rename = "Uris")]
    uris: UserUris,
}

impl User {
    const BASE_URI: &'static str = "/api/v2/user/";

    /// Returns information for the user at the provided full url
    pub async fn from_url(client: Client, url: &str) -> Result<User, SmugMugError> {
        obj_from_url!(client, url, UserResponse, user)
    }

    /// Returns information for the specified user id
    pub async fn from_id(client: Client, id: &str) -> Result<User, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?
            .join(Self::BASE_URI)?
            .join(id)?;
        Self::from_url(client, req_url.as_str()).await
    }

    /// Returns information for the authenticated user
    pub async fn authenticated_user_info(client: Client) -> Result<User, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?.join("/api/v2!authuser")?;
        Self::from_url(client, req_url.as_str()).await
    }

    /// Retrieves the root node information for this user.
    ///
    /// NOTE: if this object was deserialized externally this will fail as the internal client
    /// isn't valid
    pub async fn node(self) -> Result<Node, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?.join(self.uris.node.as_str())?;

        Node::from_url(
            self.client.ok_or(SmugMugError::ClientNotFound())?.clone(),
            req_url.as_str(),
        )
            .await
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

// Expected response from a User request
#[derive(Deserialize, Debug)]
struct UserResponse {
    #[serde(rename = "User")]
    user: User,
}
