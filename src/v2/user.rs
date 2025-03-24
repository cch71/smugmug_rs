/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::errors::SmugMugError;
use crate::v2::{ApiClient, Node, NodeResponse, API_ORIGIN};
use serde::Deserialize;
use std::sync::Arc;

/// Holds information returned from the User API
#[derive(Deserialize, Debug)]
pub struct User {
    #[serde(skip)]
    pub(crate) api_client: Arc<ApiClient>,

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

    #[serde(rename = "WebUri")]
    pub web_uri: String,

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
